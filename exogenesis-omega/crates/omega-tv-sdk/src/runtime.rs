//! Runtime management utilities

use tokio::runtime::{Builder, Runtime};

/// Create a new Tokio runtime for the SDK
pub fn create_runtime() -> Result<Runtime, std::io::Error> {
    Builder::new_multi_thread()
        .enable_all()
        .thread_name("omega-sdk")
        .worker_threads(2) // Minimal threads for TV
        .max_blocking_threads(2)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = create_runtime();
        assert!(runtime.is_ok());
    }
}
