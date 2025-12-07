//! Platform-specific webhook handlers

pub mod generic;
pub mod netflix;

pub use generic::GenericWebhookHandler;
pub use netflix::NetflixWebhookHandler;
