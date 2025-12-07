pub mod admin;
pub mod api_keys;
pub mod email;
pub mod error;
pub mod handlers;
pub mod jwt;
pub mod mfa;
pub mod middleware;
pub mod oauth;
pub mod parental;
pub mod password_reset;
pub mod profile;
pub mod rate_limit_admin_handlers;
pub mod rate_limit_config;
pub mod rbac;
pub mod scopes;
pub mod server;
pub mod session;
pub mod storage;
pub mod token;
pub mod token_family;
pub mod user;

#[cfg(test)]
mod tests;

pub use admin::{
    delete_user, get_user_detail, impersonate_user, list_users, update_user, AdminMiddleware,
    AdminUpdateUserRequest, ImpersonationTokenResponse, ListUsersQuery, ListUsersResponse,
    UserDetail, UserDetailResponse, UserListItem,
};
pub use api_keys::{ApiKey, ApiKeyManager, CreateApiKeyRequest};
pub use email::{ConsoleProvider, EmailConfig, EmailManager, EmailService, SendGridProvider};
pub use error::{AuthError, Result};
pub use handlers::{
    login, register, resend_verification, verify_email, LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse, ResendVerificationRequest, ResendVerificationResponse,
    VerifyEmailRequest, VerifyEmailResponse,
};
pub use jwt::{Claims, JwtManager};
pub use mfa::{MfaEnrollment, MfaManager};
pub use middleware::AuthMiddleware;
pub use oauth::{OAuthConfig, OAuthManager};
pub use parental::{
    ContentRating, ParentalControls, SetParentalControlsRequest, SetParentalControlsResponse,
    VerifyPinRequest, VerifyPinResponse,
};
pub use password_reset::{
    ForgotPasswordRequest, ForgotPasswordResponse, PasswordResetToken, ResetPasswordRequest,
    ResetPasswordResponse,
};
pub use profile::{
    delete_current_user, get_current_user, update_current_user, upload_avatar, ProfileStorage,
    UserProfile,
};
pub use rate_limit_admin_handlers::{
    delete_rate_limit, get_rate_limit, list_rate_limits, update_rate_limit,
    DeleteRateLimitConfigQuery, ListRateLimitConfigsResponse, RateLimitConfigResponse,
    UpdateRateLimitConfigRequest,
};
pub use rate_limit_config::{RateLimitConfig as RateLimitConfigV2, RateLimitConfigStore, UserTier};
pub use rbac::{Permission, RbacManager, Role};
pub use scopes::{Scope, ScopeManager};
pub use server::start_server;
pub use session::{Session, SessionManager};
pub use storage::AuthStorage;
pub use token::{TokenManager, TokenType};
pub use token_family::{TokenFamily, TokenFamilyManager};
pub use user::{
    CreateUserRequest, PasswordHasher, PostgresUserRepository, User, UserRepository, UserResponse,
};
