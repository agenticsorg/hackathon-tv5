/**
 * @file omega_sdk.h
 * @brief Omega TV SDK - C API for TV manufacturer integration
 * @version 0.1.0
 *
 * This SDK provides recommendation and learning capabilities for TV platforms.
 *
 * Features:
 * - On-device AI recommendations (<15ms latency)
 * - Federated learning with privacy preservation
 * - JSON-based data exchange
 * - Thread-safe API
 *
 * Example Usage:
 * @code
 * #include "omega_sdk.h"
 *
 * int main() {
 *     // Initialize SDK
 *     if (omega_init("/data/omega", "https://constellation.example.com") != OMEGA_SUCCESS) {
 *         fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
 *         return 1;
 *     }
 *
 *     // Get recommendations
 *     const char* context = "{\"genre\":\"action\",\"time\":\"evening\"}";
 *     char buffer[8192];
 *     if (omega_recommend(context, buffer, sizeof(buffer)) == OMEGA_SUCCESS) {
 *         printf("Recommendations: %s\n", buffer);
 *     }
 *
 *     // Record viewing event
 *     const char* event = "{\"content_id\":\"movie123\",\"watch_percentage\":0.85}";
 *     omega_observe(event);
 *
 *     // Sync with constellation (optional, can be done periodically)
 *     omega_sync();
 *
 *     // Cleanup
 *     omega_shutdown();
 *     return 0;
 * }
 * @endcode
 */

#ifndef OMEGA_SDK_H
#define OMEGA_SDK_H

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Error Codes
 * ============================================================================ */

/** Operation successful */
#define OMEGA_SUCCESS 0

/** Initialization failed */
#define OMEGA_ERR_INIT -1

/** Recommendation generation failed */
#define OMEGA_ERR_RECOMMEND -2

/** Observation recording failed */
#define OMEGA_ERR_OBSERVE -3

/** Synchronization with constellation failed */
#define OMEGA_ERR_SYNC -4

/** Invalid argument provided */
#define OMEGA_ERR_INVALID_ARG -5

/** JSON parsing error */
#define OMEGA_ERR_JSON_PARSE -6

/** Output buffer too small */
#define OMEGA_ERR_BUFFER_TOO_SMALL -7

/** SDK not initialized (call omega_init first) */
#define OMEGA_ERR_NOT_INITIALIZED -8

/** SDK already initialized */
#define OMEGA_ERR_ALREADY_INITIALIZED -9

/** Internal error */
#define OMEGA_ERR_INTERNAL -10

/* ============================================================================
 * Core API
 * ============================================================================ */

/**
 * @brief Initialize the Omega SDK
 *
 * Must be called once before any other SDK functions.
 * Creates the recommendation engine and establishes connection to constellation server.
 *
 * @param storage_path Path to local storage directory (e.g., "/data/omega")
 * @param constellation_url URL of constellation server (e.g., "https://constellation.example.com")
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * @note This function may take 100-500ms to complete initialization
 * @warning Call omega_shutdown() before program exit
 */
int omega_init(const char* storage_path, const char* constellation_url);

/**
 * @brief Get content recommendations based on viewing context
 *
 * Returns personalized recommendations in <15ms using on-device AI.
 *
 * @param context_json JSON string with viewing context
 * @param out_json Output buffer for JSON recommendations
 * @param out_len Size of output buffer in bytes
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * Context JSON format:
 * @code{.json}
 * {
 *   "genre": "action",           // Optional: current genre preference
 *   "time": "evening",           // Optional: time of day
 *   "device": "main_tv",         // Optional: device identifier
 *   "user_profile": "family",    // Optional: viewing profile
 *   "language": "en",            // Optional: content language
 *   "current_content": "movie123" // Optional: what's currently playing
 * }
 * @endcode
 *
 * Response JSON format:
 * @code{.json}
 * [
 *   {
 *     "content_id": "movie456",
 *     "title": "Action Movie",
 *     "score": 0.95,
 *     "reason": "Based on your viewing history"
 *   },
 *   ...
 * ]
 * @endcode
 *
 * @note Recommended buffer size: 8192 bytes
 * @warning Buffer must be large enough or OMEGA_ERR_BUFFER_TOO_SMALL is returned
 */
int omega_recommend(const char* context_json, char* out_json, int out_len);

/**
 * @brief Record a viewing event for learning
 *
 * Records user viewing behavior to improve recommendations.
 * This data is processed locally and only high-quality patterns are synced.
 *
 * @param event_json JSON string with viewing event data
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * Event JSON format:
 * @code{.json}
 * {
 *   "content_id": "movie123",      // Required: content identifier
 *   "watch_percentage": 0.85,      // Required: 0.0-1.0 (85% watched)
 *   "session_id": "session_abc",   // Optional: viewing session
 *   "duration_seconds": 3600,      // Optional: watch duration
 *   "user_rating": 4.5,            // Optional: 0.0-5.0
 *   "timestamp": "2025-12-06T10:30:00Z" // Optional: ISO 8601
 * }
 * @endcode
 *
 * @note This function returns quickly (<5ms), processing happens asynchronously
 * @note Privacy: Only aggregated patterns are synced, never raw viewing data
 */
int omega_observe(const char* event_json);

/**
 * @brief Synchronize with constellation server
 *
 * Pushes local high-quality patterns (~1KB) and receives global patterns (~5KB).
 * This should be called periodically (e.g., every 10 minutes) when network is available.
 *
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * @note This function may take 1-5 seconds depending on network
 * @note Sync is optional - recommendations work offline
 * @note Recommended to call from background thread
 */
int omega_sync(void);

/**
 * @brief Shutdown SDK and release resources
 *
 * Must be called before program exit to properly clean up resources.
 * After calling this, omega_init() must be called again before using the SDK.
 *
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * @note This function may take 100-200ms to complete cleanup
 */
int omega_shutdown(void);

/* ============================================================================
 * Utility Functions
 * ============================================================================ */

/**
 * @brief Get last error message
 *
 * Returns a human-readable error message for the last error that occurred.
 * The message is thread-local and valid until the next error or thread exit.
 *
 * @return Pointer to null-terminated error string, or NULL if no error
 *
 * @note Do not free the returned pointer
 * @note The string may be overwritten by subsequent SDK calls
 */
const char* omega_get_last_error(void);

/**
 * @brief Get SDK version string
 *
 * @return Pointer to null-terminated version string (e.g., "0.1.0")
 *
 * @note The returned pointer is always valid and points to static memory
 */
const char* omega_version(void);

/**
 * @brief Check if SDK is initialized
 *
 * @return 1 if initialized, 0 if not initialized
 */
int omega_is_initialized(void);

/* ============================================================================
 * Advanced API (Optional)
 * ============================================================================ */

/**
 * @brief Get SDK statistics
 *
 * Returns runtime statistics in JSON format.
 *
 * @param out_json Output buffer for JSON statistics
 * @param out_len Size of output buffer
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * Statistics JSON format:
 * @code{.json}
 * {
 *   "patterns_learned": 1234,
 *   "recommendations_served": 5678,
 *   "avg_latency_ms": 12.5,
 *   "last_sync": "2025-12-06T10:30:00Z",
 *   "storage_mb": 45.2
 * }
 * @endcode
 */
int omega_get_stats(char* out_json, int out_len);

/**
 * @brief Clear all local data
 *
 * Deletes all learned patterns and resets to initial state.
 * Use with caution - this cannot be undone.
 *
 * @return OMEGA_SUCCESS on success, error code on failure
 *
 * @warning This will delete all local learning data
 * @note SDK must be initialized
 */
int omega_clear_data(void);

#ifdef __cplusplus
}
#endif

#endif /* OMEGA_SDK_H */
