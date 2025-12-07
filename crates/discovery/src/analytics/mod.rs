pub mod query_log;
pub mod search_analytics;

pub use query_log::{QueryLog, SearchClick, SearchEvent};
pub use search_analytics::{
    AnalyticsDashboard, LatencyStats, PeriodType, PopularQuery, SearchAnalytics, ZeroResultQuery,
};
