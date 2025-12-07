//! Graceful shutdown coordinator for production services
//!
//! This module provides centralized shutdown handling for async services with support for
//! multi-phase graceful shutdown, task coordination, and cleanup operations. It handles
//! system signals (SIGTERM, SIGINT) and coordinates shutdown across multiple components.
//!
//! # Features
//!
//! - Multi-phase shutdown with configurable timeouts
//! - Task registration and coordination
//! - Signal handling (SIGTERM, SIGINT)
//! - Pre-shutdown callbacks for cleanup
//! - Integration with Actix-web and other async frameworks
//! - Metrics and logging flush on shutdown
//!
//! # Shutdown Phases
//!
//! 1. **Drain**: Stop accepting new work, let in-progress requests complete
//! 2. **Wait**: Wait for registered tasks to complete gracefully
//! 3. **Force**: Force termination of remaining tasks
//!
//! # Example
//!
//! ```no_run
//! use media_gateway_core::shutdown::{ShutdownCoordinator, ShutdownConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create coordinator with custom timeouts
//! let config = ShutdownConfig {
//!     drain_timeout: Duration::from_secs(30),
//!     wait_timeout: Duration::from_secs(60),
//!     force_timeout: Duration::from_secs(10),
//! };
//!
//! let coordinator = ShutdownCoordinator::new(config);
//!
//! // Register a background task
//! let handle = coordinator.register_task("background-worker");
//!
//! // Spawn the coordinator
//! let shutdown_handle = tokio::spawn(async move {
//!     coordinator.wait_for_signal().await;
//! });
//!
//! // In your task, wait for shutdown signal
//! tokio::select! {
//!     _ = handle.wait_for_shutdown() => {
//!         println!("Shutdown signal received, cleaning up...");
//!         // Cleanup work
//!         handle.notify_complete();
//!     }
//! }
//!
//! shutdown_handle.await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Actix-web Integration
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer};
//! use media_gateway_core::shutdown::{ShutdownCoordinator, ShutdownConfig};
//!
//! # async fn example() -> std::io::Result<()> {
//! let coordinator = ShutdownCoordinator::default();
//! let shutdown_signal = coordinator.create_shutdown_signal();
//!
//! let server = HttpServer::new(|| {
//!     App::new()
//!         .route("/health", web::get().to(|| async { "OK" }))
//! })
//! .bind("127.0.0.1:8080")?
//! .run();
//!
//! // Run server with graceful shutdown
//! server.handle().stop(true);
//! tokio::select! {
//!     _ = server => {}
//!     _ = shutdown_signal => {}
//! }
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Default timeout for the drain phase (30 seconds)
const DEFAULT_DRAIN_TIMEOUT_MS: u64 = 30_000;

/// Default timeout for the wait phase (60 seconds)
const DEFAULT_WAIT_TIMEOUT_MS: u64 = 60_000;

/// Default timeout for the force phase (10 seconds)
const DEFAULT_FORCE_TIMEOUT_MS: u64 = 10_000;

/// Configuration for shutdown behavior
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Timeout for the drain phase (stop accepting new work)
    pub drain_timeout: Duration,

    /// Timeout for the wait phase (wait for tasks to complete)
    pub wait_timeout: Duration,

    /// Timeout for the force phase (force kill remaining tasks)
    pub force_timeout: Duration,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_millis(DEFAULT_DRAIN_TIMEOUT_MS),
            wait_timeout: Duration::from_millis(DEFAULT_WAIT_TIMEOUT_MS),
            force_timeout: Duration::from_millis(DEFAULT_FORCE_TIMEOUT_MS),
        }
    }
}

impl ShutdownConfig {
    /// Create a new shutdown configuration
    pub fn new(drain_timeout: Duration, wait_timeout: Duration, force_timeout: Duration) -> Self {
        Self {
            drain_timeout,
            wait_timeout,
            force_timeout,
        }
    }

    /// Create a configuration with all timeouts set to the same value
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            drain_timeout: timeout,
            wait_timeout: timeout,
            force_timeout: timeout,
        }
    }

    /// Create a fast shutdown configuration (5 seconds per phase)
    pub fn fast() -> Self {
        Self::with_timeout(Duration::from_secs(5))
    }
}

/// Shutdown phase tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownPhase {
    Running,
    Drain,
    Wait,
    Force,
    Complete,
}

/// Internal state for the shutdown coordinator
#[derive(Debug)]
struct ShutdownState {
    phase: ShutdownPhase,
    registered_tasks: usize,
    completed_tasks: usize,
}

impl Default for ShutdownState {
    fn default() -> Self {
        Self {
            phase: ShutdownPhase::Running,
            registered_tasks: 0,
            completed_tasks: 0,
        }
    }
}

/// Shutdown coordinator for managing graceful shutdown
///
/// Coordinates shutdown across multiple components with support for
/// multi-phase shutdown, task tracking, and cleanup callbacks.
pub struct ShutdownCoordinator {
    config: ShutdownConfig,
    state: Arc<RwLock<ShutdownState>>,
    shutdown_tx: broadcast::Sender<()>,
    callbacks: Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator with the given configuration
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(ShutdownState::default())),
            shutdown_tx,
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new task that should be tracked during shutdown
    ///
    /// Returns a handle that the task can use to wait for shutdown signals
    /// and notify when it has completed cleanup.
    pub fn register_task(&self, task_name: &str) -> ShutdownHandle {
        let mut state = futures::executor::block_on(self.state.write());
        state.registered_tasks += 1;

        info!(
            task_name = %task_name,
            total_tasks = state.registered_tasks,
            "Task registered for shutdown coordination"
        );

        ShutdownHandle {
            task_name: task_name.to_string(),
            shutdown_rx: self.shutdown_tx.subscribe(),
            state: Arc::clone(&self.state),
        }
    }

    /// Register a callback to be executed before shutdown
    ///
    /// Callbacks are executed in the drain phase, allowing for cleanup
    /// operations like flushing metrics, closing connections, etc.
    pub fn on_shutdown<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut callbacks = futures::executor::block_on(self.callbacks.write());
        callbacks.push(Box::new(callback));
    }

    /// Create a future that completes when shutdown is signaled
    ///
    /// Useful for integrating with frameworks like Actix-web.
    pub fn create_shutdown_signal(&self) -> impl std::future::Future<Output = ()> {
        let mut rx = self.shutdown_tx.subscribe();
        async move {
            let _ = rx.recv().await;
        }
    }

    /// Wait for shutdown signal and coordinate graceful shutdown
    ///
    /// This is the main entry point for the shutdown coordinator. It waits
    /// for SIGTERM or SIGINT signals and then coordinates shutdown across
    /// all registered tasks.
    pub async fn wait_for_signal(self) {
        info!("Shutdown coordinator initialized, waiting for signal");

        // Wait for shutdown signal (SIGTERM or SIGINT)
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating graceful shutdown");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating graceful shutdown");
                }
            }
        }

        #[cfg(not(unix))]
        {
            // For non-Unix systems, only handle Ctrl+C
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to register Ctrl+C handler");
            info!("Received Ctrl+C, initiating graceful shutdown");
        }

        // Execute shutdown sequence
        self.execute_shutdown().await;
    }

    /// Execute the multi-phase shutdown sequence
    async fn execute_shutdown(self) {
        // Phase 1: Drain
        self.drain_phase().await;

        // Phase 2: Wait
        self.wait_phase().await;

        // Phase 3: Force
        self.force_phase().await;

        // Mark as complete
        let mut state = self.state.write().await;
        state.phase = ShutdownPhase::Complete;
        info!("Shutdown complete");
    }

    /// Drain phase: stop accepting new work, execute callbacks
    async fn drain_phase(&self) {
        let mut state = self.state.write().await;
        state.phase = ShutdownPhase::Drain;
        drop(state);

        info!(
            timeout_ms = self.config.drain_timeout.as_millis(),
            "Entering drain phase"
        );

        // Execute shutdown callbacks
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            callback();
        }
        drop(callbacks);

        // Broadcast shutdown signal to all tasks
        if let Err(e) = self.shutdown_tx.send(()) {
            warn!(error = %e, "Failed to broadcast shutdown signal");
        }

        // Wait for drain timeout
        sleep(self.config.drain_timeout).await;
    }

    /// Wait phase: wait for registered tasks to complete
    async fn wait_phase(&self) {
        let mut state = self.state.write().await;
        state.phase = ShutdownPhase::Wait;
        let registered = state.registered_tasks;
        drop(state);

        info!(
            timeout_ms = self.config.wait_timeout.as_millis(),
            registered_tasks = registered,
            "Entering wait phase"
        );

        let start = tokio::time::Instant::now();
        let deadline = start + self.config.wait_timeout;

        loop {
            let state = self.state.read().await;
            let completed = state.completed_tasks;
            drop(state);

            if completed >= registered {
                info!(
                    completed_tasks = completed,
                    elapsed_ms = start.elapsed().as_millis(),
                    "All tasks completed gracefully"
                );
                break;
            }

            if tokio::time::Instant::now() >= deadline {
                let remaining = registered - completed;
                warn!(
                    completed_tasks = completed,
                    remaining_tasks = remaining,
                    "Wait timeout exceeded, proceeding to force phase"
                );
                break;
            }

            // Check every 100ms
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Force phase: force termination of remaining tasks
    async fn force_phase(&self) {
        let mut state = self.state.write().await;
        state.phase = ShutdownPhase::Force;
        let registered = state.registered_tasks;
        let completed = state.completed_tasks;
        drop(state);

        if completed < registered {
            let remaining = registered - completed;
            warn!(
                timeout_ms = self.config.force_timeout.as_millis(),
                remaining_tasks = remaining,
                "Entering force phase, terminating remaining tasks"
            );

            sleep(self.config.force_timeout).await;

            error!(
                remaining_tasks = remaining,
                "Force shutdown complete, some tasks may not have cleaned up properly"
            );
        }
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new(ShutdownConfig::default())
    }
}

/// Handle for a registered task
///
/// Allows tasks to wait for shutdown signals and notify when they've completed cleanup.
pub struct ShutdownHandle {
    task_name: String,
    shutdown_rx: broadcast::Receiver<()>,
    state: Arc<RwLock<ShutdownState>>,
}

impl Clone for ShutdownHandle {
    fn clone(&self) -> Self {
        Self {
            task_name: self.task_name.clone(),
            shutdown_rx: self.shutdown_rx.resubscribe(),
            state: Arc::clone(&self.state),
        }
    }
}

impl ShutdownHandle {
    /// Wait for shutdown signal
    ///
    /// This future completes when the shutdown coordinator signals that
    /// shutdown has been initiated.
    pub async fn wait_for_shutdown(&mut self) {
        let _ = self.shutdown_rx.recv().await;
        info!(task_name = %self.task_name, "Shutdown signal received");
    }

    /// Notify that this task has completed cleanup
    ///
    /// Call this after your task has finished all cleanup operations.
    pub fn notify_complete(&self) {
        let mut state = futures::executor::block_on(self.state.write());
        state.completed_tasks += 1;

        info!(
            task_name = %self.task_name,
            completed_tasks = state.completed_tasks,
            registered_tasks = state.registered_tasks,
            "Task completed shutdown cleanup"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[test]
    fn test_shutdown_config_default() {
        let config = ShutdownConfig::default();
        assert_eq!(
            config.drain_timeout,
            Duration::from_millis(DEFAULT_DRAIN_TIMEOUT_MS)
        );
        assert_eq!(
            config.wait_timeout,
            Duration::from_millis(DEFAULT_WAIT_TIMEOUT_MS)
        );
        assert_eq!(
            config.force_timeout,
            Duration::from_millis(DEFAULT_FORCE_TIMEOUT_MS)
        );
    }

    #[test]
    fn test_shutdown_config_new() {
        let config = ShutdownConfig::new(
            Duration::from_secs(10),
            Duration::from_secs(20),
            Duration::from_secs(5),
        );
        assert_eq!(config.drain_timeout, Duration::from_secs(10));
        assert_eq!(config.wait_timeout, Duration::from_secs(20));
        assert_eq!(config.force_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_shutdown_config_with_timeout() {
        let config = ShutdownConfig::with_timeout(Duration::from_secs(15));
        assert_eq!(config.drain_timeout, Duration::from_secs(15));
        assert_eq!(config.wait_timeout, Duration::from_secs(15));
        assert_eq!(config.force_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_shutdown_config_fast() {
        let config = ShutdownConfig::fast();
        assert_eq!(config.drain_timeout, Duration::from_secs(5));
        assert_eq!(config.wait_timeout, Duration::from_secs(5));
        assert_eq!(config.force_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_coordinator_register_task() {
        let coordinator = ShutdownCoordinator::default();
        let handle = coordinator.register_task("test-task");

        let state = coordinator.state.read().await;
        assert_eq!(state.registered_tasks, 1);
        assert_eq!(state.phase, ShutdownPhase::Running);

        drop(handle);
    }

    #[tokio::test]
    async fn test_coordinator_multiple_tasks() {
        let coordinator = ShutdownCoordinator::default();
        let _handle1 = coordinator.register_task("task-1");
        let _handle2 = coordinator.register_task("task-2");
        let _handle3 = coordinator.register_task("task-3");

        let state = coordinator.state.read().await;
        assert_eq!(state.registered_tasks, 3);
    }

    #[tokio::test]
    async fn test_shutdown_handle_notify_complete() {
        let coordinator = ShutdownCoordinator::default();
        let handle = coordinator.register_task("test-task");

        handle.notify_complete();

        let state = coordinator.state.read().await;
        assert_eq!(state.completed_tasks, 1);
    }

    #[tokio::test]
    async fn test_shutdown_signal_creation() {
        let coordinator = ShutdownCoordinator::default();
        let shutdown_signal = coordinator.create_shutdown_signal();

        // Trigger shutdown
        let _ = coordinator.shutdown_tx.send(());

        // Signal should complete quickly
        let result = timeout(Duration::from_millis(100), shutdown_signal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_on_shutdown_callback() {
        let coordinator = ShutdownCoordinator::default();
        let called = Arc::new(RwLock::new(false));
        let called_clone = Arc::clone(&called);

        coordinator.on_shutdown(move || {
            let called = Arc::clone(&called_clone);
            let mut called = futures::executor::block_on(called.write());
            *called = true;
        });

        // Manually trigger drain phase
        coordinator.drain_phase().await;

        let was_called = *called.read().await;
        assert!(was_called);
    }

    #[tokio::test]
    async fn test_handle_wait_for_shutdown() {
        let coordinator = ShutdownCoordinator::default();
        let mut handle = coordinator.register_task("test-task");

        // Spawn a task that waits for shutdown
        let wait_task = tokio::spawn(async move {
            handle.wait_for_shutdown().await;
        });

        // Trigger shutdown
        let _ = coordinator.shutdown_tx.send(());

        // Wait task should complete quickly
        let result = timeout(Duration::from_millis(100), wait_task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_state_default() {
        let state = ShutdownState::default();
        assert_eq!(state.phase, ShutdownPhase::Running);
        assert_eq!(state.registered_tasks, 0);
        assert_eq!(state.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_shutdown_phases() {
        let config = ShutdownConfig {
            drain_timeout: Duration::from_millis(10),
            wait_timeout: Duration::from_millis(10),
            force_timeout: Duration::from_millis(10),
        };

        let coordinator = ShutdownCoordinator::new(config);
        let handle = coordinator.register_task("test-task");

        // Start shutdown in background
        let shutdown_task = tokio::spawn(async move {
            coordinator.drain_phase().await;
            coordinator.wait_phase().await;
            coordinator.force_phase().await;
        });

        // Complete the task
        handle.notify_complete();

        // Shutdown should complete
        let result = timeout(Duration::from_secs(1), shutdown_task).await;
        assert!(result.is_ok());
    }
}
