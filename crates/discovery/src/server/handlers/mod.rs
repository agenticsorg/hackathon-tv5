pub mod analytics;
pub mod quality;
pub mod ranking;
pub mod search;

pub use analytics::get_analytics;
pub use quality::get_quality_report;
pub use ranking::{
    delete_ranking_variant, get_ranking_config, get_ranking_config_history, get_ranking_variant,
    list_ranking_variants, update_ranking_config, update_ranking_variant,
};
pub use search::{autocomplete, execute_search};
