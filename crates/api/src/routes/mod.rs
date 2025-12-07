pub mod content;
pub mod discover;
pub mod playback;
pub mod search;
pub mod sona;
pub mod sync;
pub mod user;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(content::configure)
            .configure(search::configure)
            .configure(discover::configure)
            .configure(user::configure)
            .configure(sona::configure)
            .configure(playback::configure)
            .configure(sync::configure),
    );
}
