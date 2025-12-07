// Temporary patch file - manual integration needed

// Add to imports:
use crate::parental::{
    update_parental_controls, verify_parental_pin, ParentalControlsState,
};

// Add after profile_state creation (around line 865):
let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

let parental_state = Data::new(ParentalControlsState {
    db_pool: db_pool.clone(),
    redis_client: redis_client.clone(),
    jwt_secret,
});

// Add to HttpServer app_data (after profile_state.clone()):
.app_data(parental_state.clone())

// Add to service registrations (after mfa_challenge):
.service(update_parental_controls)
.service(verify_parental_pin)
