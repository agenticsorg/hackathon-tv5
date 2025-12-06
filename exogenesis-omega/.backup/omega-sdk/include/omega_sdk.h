/**
 * @file omega_sdk.h
 * @brief Omega SDK - C API for Exogenesis Omega TV Recommendation System
 *
 * @copyright Copyright (c) 2024 Exogenesis Omega
 * @license MIT
 *
 * ## Overview
 *
 * The Omega SDK provides a simple C API for integrating intelligent,
 * privacy-preserving recommendations into TV platforms. All inference
 * happens locally on the TV, with periodic pattern synchronization.
 *
 * ## Quick Start
 *
 * ```c
 * // 1. Initialize on TV boot
 * int result = omega_init(
 *     "/data/omega/model.onnx",
 *     "/data/omega/brain.db"
 * );
 * if (result != 0) {
 *     fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
 *     return -1;
 * }
 *
 * // 2. Get recommendations when user browses
 * char recommendations[4096];
 * omega_recommend(
 *     "{\"time\":\"evening\",\"mood\":\"relaxed\"}",
 *     recommendations,
 *     sizeof(recommendations)
 * );
 * printf("Recommendations: %s\n", recommendations);
 *
 * // 3. Record viewing events
 * omega_observe("{\"content_id\":\"movie123\",\"watch_pct\":0.95}");
 *
 * // 4. Sync every 5-15 minutes
 * omega_sync();
 *
 * // 5. Shutdown on TV power-off
 * omega_shutdown();
 * ```
 *
 * ## Performance
 *
 * - Recommendation latency: <15ms (p99)
 * - Memory footprint: ~200MB
 * - Sync bandwidth: ~1KB push, ~5KB pull per sync
 *
 * ## Error Handling
 *
 * All functions return 0 on success, negative error codes on failure.
 * Use omega_get_last_error() to get detailed error messages.
 */

#ifndef OMEGA_SDK_H
#define OMEGA_SDK_H

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * Error Codes
 * ======================================================================== */

/**
 * @defgroup ErrorCodes Error Codes
 * @{
 */

/** Success */
#define OMEGA_SUCCESS           0

/** Initialization error (model not found, storage error, etc.) */
#define OMEGA_ERR_INIT         -1

/** Inference error (ONNX runtime error, invalid context, etc.) */
#define OMEGA_ERR_INFERENCE    -2

/** Sync error (network error, constellation unreachable, etc.) */
#define OMEGA_ERR_SYNC         -3

/** Storage error (database error, disk full, etc.) */
#define OMEGA_ERR_STORAGE      -4

/** Invalid input (null pointer, invalid JSON, etc.) */
#define OMEGA_ERR_INVALID      -5

/** Internal error (unexpected state, bug in SDK, etc.) */
#define OMEGA_ERR_INTERNAL     -99

/** @} */

/* ========================================================================
 * Core Functions
 * ======================================================================== */

/**
 * @brief Initialize Omega Brain on TV boot
 *
 * This function must be called once during TV startup before any other
 * omega_* functions. It loads the ONNX model, initializes the vector
 * database, and prepares the recommendation engine.
 *
 * @param model_path Path to ONNX model file (e.g., "/data/omega/model.onnx")
 *                   The model should be a 4-bit quantized MiniLM model (~100MB)
 * @param storage_path Path for persistent storage (e.g., "/data/omega/brain.db")
 *                     The SDK will create this directory if it doesn't exist
 *
 * @return 0 on success, negative error code on failure
 *
 * @note This function may take 1-2 seconds to complete as it loads the model
 * @note The function is NOT thread-safe. Call only once from main thread.
 *
 * @example
 * ```c
 * int result = omega_init("/data/omega/model.onnx", "/data/omega/brain.db");
 * if (result != 0) {
 *     const char* error = omega_get_last_error();
 *     fprintf(stderr, "Omega init failed: %s (code: %d)\n", error, result);
 *     omega_free_error(error);
 *     return -1;
 * }
 * printf("Omega initialized successfully\n");
 * ```
 */
int omega_init(const char* model_path, const char* storage_path);

/**
 * @brief Get personalized recommendations
 *
 * Returns recommendations based on the current viewing context. This function
 * is designed to be very fast (<15ms) as it runs entirely on the TV using
 * local inference and vector search.
 *
 * @param context_json JSON string describing the viewing context
 *                     Example: {"time":"evening","mood":"relaxed","recent":["action","thriller"]}
 *                     Supported fields:
 *                     - time: "morning"|"afternoon"|"evening"|"night"
 *                     - mood: "relaxed"|"energetic"|"focused"|"bored"
 *                     - recent: array of recently watched content IDs or genres
 * @param out_json Output buffer for JSON recommendations array
 *                 Format: [{"content_id":"abc","score":0.95,"metadata":{...}},...]
 * @param out_len Size of output buffer in bytes (recommend 4KB minimum)
 *
 * @return 0 on success, negative error code on failure
 *
 * @note Function completes in <15ms (p99)
 * @note Thread-safe after initialization
 * @note If buffer is too small, returns OMEGA_ERR_INVALID
 *
 * @example
 * ```c
 * char recommendations[4096];
 * int result = omega_recommend(
 *     "{\"time\":\"evening\",\"mood\":\"relaxed\"}",
 *     recommendations,
 *     sizeof(recommendations)
 * );
 * if (result == 0) {
 *     printf("Recommendations: %s\n", recommendations);
 *     // Parse JSON and display to user
 * } else {
 *     fprintf(stderr, "Recommendation failed: %d\n", result);
 * }
 * ```
 */
int omega_recommend(
    const char* context_json,
    char* out_json,
    int out_len
);

/**
 * @brief Record viewing event
 *
 * Call this function when the user watches content. The SDK will learn from
 * these events to improve future recommendations. Events are stored locally
 * and never sent to the cloud in raw form.
 *
 * @param event_json JSON string describing the viewing event
 *                   Example: {"content_id":"movie123","watch_pct":0.95,"duration_sec":3600}
 *                   Required fields:
 *                   - content_id: unique identifier for the content
 *                   - watch_pct: percentage watched (0.0-1.0)
 *                   Optional fields:
 *                   - duration_sec: total watch duration
 *                   - rating: user rating (1-5)
 *                   - completed: boolean, true if finished
 *
 * @return 0 on success, negative error code on failure
 *
 * @note Function completes very quickly (<1ms)
 * @note Thread-safe after initialization
 * @note Can be called frequently without performance impact
 *
 * @example
 * ```c
 * // User watched 95% of a movie
 * omega_observe("{\"content_id\":\"movie123\",\"watch_pct\":0.95,\"completed\":true}");
 *
 * // User watched 30% of a series episode
 * omega_observe("{\"content_id\":\"series456_ep1\",\"watch_pct\":0.30}");
 * ```
 */
int omega_observe(const char* event_json);

/**
 * @brief Sync patterns with Omega Constellation
 *
 * Performs bidirectional sync with the Omega Constellation:
 * - Pushes local high-quality patterns (~1KB compressed)
 * - Pulls global patterns and trends (~5KB compressed)
 *
 * This function should be called every 5-15 minutes when network is available.
 * It's designed to be called in the background and won't block the UI.
 *
 * @return 0 on success, negative error code on failure
 *
 * @note Recommended interval: 5-15 minutes
 * @note Bandwidth usage: ~6KB per sync
 * @note Function is blocking; call from background thread
 * @note Safe to call even when offline (will fail gracefully)
 *
 * @example
 * ```c
 * // Call from background thread every 5 minutes
 * void* sync_thread(void* arg) {
 *     while (running) {
 *         sleep(300); // 5 minutes
 *
 *         int result = omega_sync();
 *         if (result == 0) {
 *             printf("Sync successful\n");
 *         } else if (result == OMEGA_ERR_SYNC) {
 *             printf("Sync failed (network?), will retry later\n");
 *         } else {
 *             fprintf(stderr, "Sync error: %d\n", result);
 *         }
 *     }
 *     return NULL;
 * }
 * ```
 */
int omega_sync(void);

/**
 * @brief Shutdown and persist state
 *
 * Call this function before TV shutdown to ensure all data is saved.
 * It will flush pending writes, save patterns, and cleanup resources.
 *
 * @return 0 on success, negative error code on failure
 *
 * @note Should be called during TV shutdown sequence
 * @note Blocks until all data is persisted (may take 1-2 seconds)
 * @note Safe to call multiple times
 * @note After shutdown, omega_init must be called again
 *
 * @example
 * ```c
 * // TV shutdown handler
 * void on_tv_shutdown(void) {
 *     printf("Shutting down Omega...\n");
 *     omega_shutdown();
 *     printf("Omega shutdown complete\n");
 * }
 * ```
 */
int omega_shutdown(void);

/* ========================================================================
 * Error Handling
 * ======================================================================== */

/**
 * @brief Get last error message
 *
 * Returns a human-readable description of the last error that occurred.
 * The error message is stored per-thread and is valid until the next
 * omega_* call on the same thread.
 *
 * @return Pointer to null-terminated error string, or NULL if no error
 *
 * @note The returned pointer must be freed with omega_free_error()
 * @note Thread-safe (uses thread-local storage)
 * @note Returns NULL if no error occurred
 *
 * @example
 * ```c
 * int result = omega_init("/bad/path", "/bad/path");
 * if (result != 0) {
 *     const char* error = omega_get_last_error();
 *     if (error) {
 *         fprintf(stderr, "Error: %s (code: %d)\n", error, result);
 *         omega_free_error(error);
 *     }
 * }
 * ```
 */
const char* omega_get_last_error(void);

/**
 * @brief Free error string returned by omega_get_last_error
 *
 * Frees the memory allocated for the error message. Must be called
 * for every pointer returned by omega_get_last_error().
 *
 * @param ptr Pointer returned by omega_get_last_error()
 *
 * @note Safe to call with NULL pointer
 * @note Must not be called more than once with the same pointer
 * @note Must only be called with pointers from omega_get_last_error()
 *
 * @example
 * ```c
 * const char* error = omega_get_last_error();
 * if (error) {
 *     fprintf(stderr, "Error: %s\n", error);
 *     omega_free_error(error);
 * }
 * ```
 */
void omega_free_error(const char* ptr);

/* ========================================================================
 * Constants
 * ======================================================================== */

/**
 * @defgroup Constants SDK Constants
 * @{
 */

/** Recommended sync interval in seconds (5 minutes) */
#define OMEGA_SYNC_INTERVAL_RECOMMENDED  300

/** Maximum sync interval in seconds (15 minutes) */
#define OMEGA_SYNC_INTERVAL_MAX          900

/** Minimum sync interval in seconds (1 minute) */
#define OMEGA_SYNC_INTERVAL_MIN          60

/** Recommended output buffer size for recommendations */
#define OMEGA_RECOMMEND_BUFFER_SIZE      4096

/** Expected memory footprint in MB */
#define OMEGA_MEMORY_FOOTPRINT_MB        200

/** @} */

#ifdef __cplusplus
}
#endif

#endif /* OMEGA_SDK_H */
