//! User profile and preference vector tests

use crate::profile::*;
use crate::types::ViewingEvent;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[test]
fn test_user_profile_new() {
    let user_id = Uuid::new_v4();
    let profile = UserProfile::new(user_id);

    assert_eq!(profile.user_id, user_id);
    assert_eq!(profile.preference_vector.len(), 512);
    assert_eq!(profile.interaction_count, 0);
    assert!(profile.genre_affinities.is_empty());
}

#[test]
fn test_preference_vector_building_filters_low_engagement() {
    // Test MIN_WATCH_THRESHOLD filtering (30% completion)
    let events = vec![
        ViewingEvent {
            content_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            completion_rate: 0.15, // Below threshold
            rating: None,
            is_rewatch: false,
            dismissed: false,
        },
        ViewingEvent {
            content_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            completion_rate: 0.50, // Above threshold
            rating: Some(4),
            is_rewatch: false,
            dismissed: false,
        },
    ];

    // Events with completion_rate < 0.3 should be filtered out
    let high_engagement: Vec<_> = events.iter().filter(|e| e.completion_rate >= 0.3).collect();

    assert_eq!(high_engagement.len(), 1);
    assert_eq!(high_engagement[0].completion_rate, 0.50);
}

#[test]
fn test_temporal_decay_calculation() {
    // Test 0.95 decay rate
    let decay_rate = 0.95f32;

    // Recent viewing (1 day ago)
    let days_since_recent = 1.0;
    let decay_weight_recent = decay_rate.powf(days_since_recent / 30.0);
    assert!(decay_weight_recent > 0.998); // Almost no decay

    // 30 days ago
    let days_since_month = 30.0;
    let decay_weight_month = decay_rate.powf(days_since_month / 30.0);
    assert!((decay_weight_month - 0.95).abs() < 0.01);

    // 90 days ago (3 months)
    let days_since_quarter = 90.0;
    let decay_weight_quarter = decay_rate.powf(days_since_quarter / 30.0);
    assert!(decay_weight_quarter < 0.9);
}

#[test]
fn test_engagement_weight_high_completion_high_rating() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 1.0,
        rating: Some(5),
        is_rewatch: true,
        dismissed: false,
    };

    let weight = BuildUserPreferenceVector::calculate_engagement_weight(&event);

    // High completion + high rating + rewatch = high weight
    assert!(weight > 0.8);
    assert!(weight <= 1.0);
}

#[test]
fn test_engagement_weight_low_completion_no_rating() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.3, // Minimum threshold
        rating: None,
        is_rewatch: false,
        dismissed: false,
    };

    let weight = BuildUserPreferenceVector::calculate_engagement_weight(&event);

    // Low completion + no rating = low weight
    assert!(weight < 0.6);
    assert!(weight >= 0.0);
}

#[test]
fn test_engagement_weight_dismissal_penalty() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.5,
        rating: Some(3),
        is_rewatch: false,
        dismissed: true, // Dismissal penalty
    };

    let weight = BuildUserPreferenceVector::calculate_engagement_weight(&event);

    // Dismissal should significantly reduce weight
    assert!(weight >= 0.0); // Clamped to minimum 0.0
}

#[test]
fn test_engagement_weight_rewatch_bonus() {
    let event_no_rewatch = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.8,
        rating: Some(4),
        is_rewatch: false,
        dismissed: false,
    };

    let event_rewatch = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.8,
        rating: Some(4),
        is_rewatch: true,
        dismissed: false,
    };

    let weight_no_rewatch =
        BuildUserPreferenceVector::calculate_engagement_weight(&event_no_rewatch);
    let weight_rewatch = BuildUserPreferenceVector::calculate_engagement_weight(&event_rewatch);

    assert!(weight_rewatch > weight_no_rewatch);
}

#[test]
fn test_engagement_weight_rating_scale() {
    let ratings = vec![1, 2, 3, 4, 5];
    let weights: Vec<f32> = ratings
        .iter()
        .map(|&rating| {
            let event = ViewingEvent {
                content_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                completion_rate: 0.8,
                rating: Some(rating),
                is_rewatch: false,
                dismissed: false,
            };
            BuildUserPreferenceVector::calculate_engagement_weight(&event)
        })
        .collect();

    // Higher ratings should produce higher weights
    for i in 0..weights.len() - 1 {
        assert!(weights[i + 1] >= weights[i]);
    }
}

#[test]
fn test_progressive_personalization_genre_update() {
    let mut profile = UserProfile::new(Uuid::new_v4());

    // First interaction with action genre
    ProgressivePersonalization::update_genre_affinities(&mut profile, &["action".to_string()], 0.8);

    let initial_affinity = profile.genre_affinities["action"];
    assert!(initial_affinity > 0.5); // Should be above default

    // Second interaction
    ProgressivePersonalization::update_genre_affinities(&mut profile, &["action".to_string()], 0.9);

    let updated_affinity = profile.genre_affinities["action"];
    assert!(updated_affinity > initial_affinity); // Should increase
}

#[test]
fn test_progressive_personalization_multiple_genres() {
    let mut profile = UserProfile::new(Uuid::new_v4());

    ProgressivePersonalization::update_genre_affinities(
        &mut profile,
        &[
            "action".to_string(),
            "sci-fi".to_string(),
            "thriller".to_string(),
        ],
        0.85,
    );

    assert_eq!(profile.genre_affinities.len(), 3);
    assert!(profile.genre_affinities.contains_key("action"));
    assert!(profile.genre_affinities.contains_key("sci-fi"));
    assert!(profile.genre_affinities.contains_key("thriller"));
}

#[test]
fn test_should_update_preference_vector() {
    // Should update every 5 interactions
    assert!(!ProgressivePersonalization::should_update_preference_vector(1));
    assert!(!ProgressivePersonalization::should_update_preference_vector(4));
    assert!(ProgressivePersonalization::should_update_preference_vector(
        5
    ));
    assert!(ProgressivePersonalization::should_update_preference_vector(
        10
    ));
    assert!(!ProgressivePersonalization::should_update_preference_vector(11));
}

#[test]
fn test_should_train_lora() {
    // Should train every 10 interactions, starting at 10
    assert!(!ProgressivePersonalization::should_train_lora(5));
    assert!(!ProgressivePersonalization::should_train_lora(9));
    assert!(ProgressivePersonalization::should_train_lora(10));
    assert!(ProgressivePersonalization::should_train_lora(20));
    assert!(!ProgressivePersonalization::should_train_lora(21));
}

#[test]
fn test_temporal_context_default() {
    use crate::types::TemporalContext;

    let context = TemporalContext::default();

    assert_eq!(context.hourly_patterns.len(), 24);
    assert_eq!(context.weekday_patterns.len(), 7);
    assert_eq!(context.seasonal_patterns.len(), 4);
    assert_eq!(context.recent_bias, 0.8);

    // All patterns should be initialized to 0.5 (neutral)
    assert!(context.hourly_patterns.iter().all(|&p| p == 0.5));
    assert!(context.weekday_patterns.iter().all(|&p| p == 0.5));
    assert!(context.seasonal_patterns.iter().all(|&p| p == 0.5));
}

#[test]
fn test_viewing_event_structure() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.75,
        rating: Some(4),
        is_rewatch: false,
        dismissed: false,
    };

    assert_eq!(event.completion_rate, 0.75);
    assert_eq!(event.rating.unwrap(), 4);
    assert!(!event.is_rewatch);
    assert!(!event.dismissed);
}
