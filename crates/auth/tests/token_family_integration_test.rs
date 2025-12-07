/// Integration tests for token family refresh token rotation
///
/// These tests verify the security feature that detects and mitigates refresh token reuse attacks.
/// When a refresh token is reused (possibly indicating theft), the entire token family is revoked.

#[cfg(test)]
mod token_family_tests {
    use auth::{JwtManager, SessionManager, TokenFamilyManager};
    use std::sync::Arc;

    /// Helper to create test managers
    fn create_test_managers() -> (
        Arc<JwtManager>,
        Arc<TokenFamilyManager>,
        Arc<SessionManager>,
    ) {
        // For actual integration tests, use real Redis connection
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

        let token_family_manager = Arc::new(TokenFamilyManager::new(&redis_url).unwrap());
        let session_manager = Arc::new(SessionManager::new(&redis_url).unwrap());

        // Load test RSA keys
        let private_key = include_bytes!("../../tests/fixtures/test_private_key.pem");
        let public_key = include_bytes!("../../tests/fixtures/test_public_key.pem");

        let jwt_manager = Arc::new(
            JwtManager::new(
                private_key,
                public_key,
                "https://test.mediagateway.io".to_string(),
                "test-users".to_string(),
            )
            .unwrap(),
        );

        (jwt_manager, token_family_manager, session_manager)
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_token_family_creation_and_tracking() {
        let (jwt_manager, token_family_manager, _) = create_test_managers();
        let user_id = "test_user_001".to_string();

        // Create a new token family
        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();

        // Create a refresh token with this family
        let refresh_token = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();

        // Verify and extract claims
        let claims = jwt_manager.verify_refresh_token(&refresh_token).unwrap();
        assert_eq!(claims.token_family_id, Some(family_id));

        // Add token to family
        token_family_manager
            .add_token_to_family(family_id, claims.jti.clone())
            .await
            .unwrap();

        // Verify token is in family
        let is_in_family = token_family_manager
            .is_token_in_family(family_id, &claims.jti)
            .await
            .unwrap();
        assert!(is_in_family);
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_successful_token_rotation() {
        let (jwt_manager, token_family_manager, session_manager) = create_test_managers();
        let user_id = "test_user_002".to_string();

        // Initial token creation
        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();
        let token1 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();

        let claims1 = jwt_manager.verify_refresh_token(&token1).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims1.jti.clone())
            .await
            .unwrap();

        // Simulate token rotation (normal refresh flow)
        // Step 1: Verify old token is in family
        let is_valid = token_family_manager
            .is_token_in_family(family_id, &claims1.jti)
            .await
            .unwrap();
        assert!(is_valid);

        // Step 2: Remove old token from family
        token_family_manager
            .remove_token_from_family(family_id, &claims1.jti)
            .await
            .unwrap();
        session_manager
            .revoke_token(&claims1.jti, 3600)
            .await
            .unwrap();

        // Step 3: Create new token with same family
        let token2 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();

        let claims2 = jwt_manager.verify_refresh_token(&token2).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims2.jti.clone())
            .await
            .unwrap();

        // Verify: old token NOT in family, new token IS in family
        let old_in_family = token_family_manager
            .is_token_in_family(family_id, &claims1.jti)
            .await
            .unwrap();
        let new_in_family = token_family_manager
            .is_token_in_family(family_id, &claims2.jti)
            .await
            .unwrap();

        assert!(!old_in_family, "Old token should be removed from family");
        assert!(new_in_family, "New token should be in family");
        assert!(
            session_manager
                .is_token_revoked(&claims1.jti)
                .await
                .unwrap(),
            "Old token should be revoked"
        );
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_token_reuse_detection_revokes_entire_family() {
        let (jwt_manager, token_family_manager, session_manager) = create_test_managers();
        let user_id = "test_user_003".to_string();

        // Initial token creation
        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();
        let token1 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();

        let claims1 = jwt_manager.verify_refresh_token(&token1).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims1.jti.clone())
            .await
            .unwrap();

        // First rotation (legitimate)
        token_family_manager
            .remove_token_from_family(family_id, &claims1.jti)
            .await
            .unwrap();
        let token2 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();
        let claims2 = jwt_manager.verify_refresh_token(&token2).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims2.jti.clone())
            .await
            .unwrap();

        // Second rotation (legitimate)
        token_family_manager
            .remove_token_from_family(family_id, &claims2.jti)
            .await
            .unwrap();
        let token3 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();
        let claims3 = jwt_manager.verify_refresh_token(&token3).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims3.jti.clone())
            .await
            .unwrap();

        // ATTACK: Attempt to reuse token1 (which was already rotated out)
        // This simulates a stolen token being used
        let is_token1_in_family = token_family_manager
            .is_token_in_family(family_id, &claims1.jti)
            .await
            .unwrap();
        assert!(
            !is_token1_in_family,
            "Token1 should NOT be in family (was rotated out)"
        );

        // Detection: When we detect token1 is not in family, we revoke the entire family
        if !is_token1_in_family {
            token_family_manager.revoke_family(family_id).await.unwrap();
        }

        // Verify: ALL tokens in the family are now revoked
        assert!(
            session_manager
                .is_token_revoked(&claims1.jti)
                .await
                .unwrap(),
            "Token1 should be revoked"
        );
        assert!(
            session_manager
                .is_token_revoked(&claims2.jti)
                .await
                .unwrap(),
            "Token2 should be revoked"
        );
        assert!(
            session_manager
                .is_token_revoked(&claims3.jti)
                .await
                .unwrap(),
            "Token3 should be revoked"
        );

        // Verify: Family no longer exists
        let family_result = token_family_manager.get_family(family_id).await;
        assert!(
            family_result.is_err(),
            "Family should be deleted after revocation"
        );
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_token_reuse_detection_with_concurrent_clients() {
        let (jwt_manager, token_family_manager, session_manager) = create_test_managers();
        let user_id = "test_user_004".to_string();

        // Setup: User has legitimate token
        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();
        let token1 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();
        let claims1 = jwt_manager.verify_refresh_token(&token1).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims1.jti.clone())
            .await
            .unwrap();

        // Scenario: User refreshes token on legitimate device
        token_family_manager
            .remove_token_from_family(family_id, &claims1.jti)
            .await
            .unwrap();
        session_manager
            .revoke_token(&claims1.jti, 3600)
            .await
            .unwrap();
        let token2 = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();
        let claims2 = jwt_manager.verify_refresh_token(&token2).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims2.jti.clone())
            .await
            .unwrap();

        // ATTACK: Attacker tries to use stolen token1
        let is_valid = token_family_manager
            .is_token_in_family(family_id, &claims1.jti)
            .await
            .unwrap();
        assert!(!is_valid, "Stolen token should not be valid");

        // Also verify it's in revoked list
        assert!(session_manager
            .is_token_revoked(&claims1.jti)
            .await
            .unwrap());

        // Revoke entire family upon detection
        token_family_manager.revoke_family(family_id).await.unwrap();

        // Verify all tokens (including legitimate user's current token) are revoked
        // This forces re-authentication, protecting the user
        assert!(session_manager
            .is_token_revoked(&claims2.jti)
            .await
            .unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_redis_performance_under_5ms() {
        let (jwt_manager, token_family_manager, _) = create_test_managers();
        let user_id = "test_user_perf".to_string();

        // Create family and token
        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();
        let token = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();
        let claims = jwt_manager.verify_refresh_token(&token).unwrap();
        token_family_manager
            .add_token_to_family(family_id, claims.jti.clone())
            .await
            .unwrap();

        // Measure is_token_in_family performance (hot path)
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = token_family_manager
                .is_token_in_family(family_id, &claims.jti)
                .await
                .unwrap();
        }
        let duration = start.elapsed();
        let avg_latency = duration.as_micros() / 100;

        println!("Average latency for is_token_in_family: {}μs", avg_latency);
        assert!(
            avg_latency < 5000,
            "Average latency should be under 5ms (5000μs), got {}μs",
            avg_latency
        );
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_multiple_token_rotations_in_family() {
        let (jwt_manager, token_family_manager, _) = create_test_managers();
        let user_id = "test_user_005".to_string();

        let family_id = token_family_manager
            .create_family(user_id.clone())
            .await
            .unwrap();

        let mut current_token = jwt_manager
            .create_refresh_token_with_family(
                user_id.clone(),
                Some("test@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
                family_id,
            )
            .unwrap();

        let mut current_claims = jwt_manager.verify_refresh_token(&current_token).unwrap();
        token_family_manager
            .add_token_to_family(family_id, current_claims.jti.clone())
            .await
            .unwrap();

        // Perform 10 rotations
        for i in 0..10 {
            // Remove old token
            token_family_manager
                .remove_token_from_family(family_id, &current_claims.jti)
                .await
                .unwrap();

            // Create new token
            current_token = jwt_manager
                .create_refresh_token_with_family(
                    user_id.clone(),
                    Some("test@example.com".to_string()),
                    vec!["user".to_string()],
                    vec!["read:content".to_string()],
                    family_id,
                )
                .unwrap();

            current_claims = jwt_manager.verify_refresh_token(&current_token).unwrap();
            token_family_manager
                .add_token_to_family(family_id, current_claims.jti.clone())
                .await
                .unwrap();

            // Verify current token is in family
            let is_valid = token_family_manager
                .is_token_in_family(family_id, &current_claims.jti)
                .await
                .unwrap();
            assert!(is_valid, "Token {} should be in family", i + 1);
        }

        // Verify family has only 1 active token (the latest one)
        let family = token_family_manager.get_family(family_id).await.unwrap();
        assert_eq!(
            family.active_jtis.len(),
            1,
            "Family should have exactly 1 active token after rotations"
        );
        assert!(
            family.active_jtis.contains(&current_claims.jti),
            "Family should contain the latest token"
        );
    }
}
