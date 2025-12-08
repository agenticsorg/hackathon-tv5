/**
 * @file simple_usage.c
 * @brief Simple example of using the Omega TV SDK
 *
 * Compile:
 *   gcc simple_usage.c -L../target/release -lomega_tv_sdk -o simple_usage
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./simple_usage
 */

#include "../include/omega_sdk.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define BUFFER_SIZE 8192

void print_error(const char* operation) {
    const char* error = omega_get_last_error();
    if (error) {
        fprintf(stderr, "Error in %s: %s\n", operation, error);
    } else {
        fprintf(stderr, "Error in %s: Unknown error\n", operation);
    }
}

int main(int argc, char** argv) {
    int ret;
    char buffer[BUFFER_SIZE];

    printf("Omega TV SDK Example\n");
    printf("====================\n\n");

    // Print version
    printf("SDK Version: %s\n\n", omega_version());

    // Check initialization state
    printf("Initialized: %s\n", omega_is_initialized() ? "Yes" : "No");

    // Initialize SDK
    printf("Initializing SDK...\n");
    ret = omega_init("/tmp/omega_example", "http://localhost:8080");
    if (ret != OMEGA_SUCCESS) {
        print_error("omega_init");
        return 1;
    }
    printf("✓ SDK initialized successfully\n\n");

    // Get recommendations
    printf("Getting recommendations...\n");
    const char* context = "{"
        "\"genre\":\"action\","
        "\"time\":\"evening\","
        "\"device\":\"main_tv\""
    "}";

    ret = omega_recommend(context, buffer, BUFFER_SIZE);
    if (ret != OMEGA_SUCCESS) {
        print_error("omega_recommend");
        goto cleanup;
    }
    printf("✓ Recommendations received:\n%s\n\n", buffer);

    // Record viewing event
    printf("Recording viewing event...\n");
    const char* event = "{"
        "\"content_id\":\"movie123\","
        "\"watch_percentage\":0.85,"
        "\"session_id\":\"session_abc\","
        "\"duration_seconds\":3600"
    "}";

    ret = omega_observe(event);
    if (ret != OMEGA_SUCCESS) {
        print_error("omega_observe");
        goto cleanup;
    }
    printf("✓ Event recorded successfully\n\n");

    // Sync with constellation
    printf("Syncing with constellation...\n");
    ret = omega_sync();
    if (ret != OMEGA_SUCCESS) {
        // Sync errors are non-fatal (might be offline)
        printf("⚠ Sync failed (this is OK if offline): %s\n\n", omega_get_last_error());
    } else {
        printf("✓ Sync completed successfully\n\n");
    }

    // Get statistics (if implemented)
    printf("Getting statistics...\n");
    ret = omega_get_stats(buffer, BUFFER_SIZE);
    if (ret == OMEGA_SUCCESS) {
        printf("✓ Statistics:\n%s\n\n", buffer);
    } else {
        printf("⚠ Statistics not available\n\n");
    }

cleanup:
    // Shutdown
    printf("Shutting down SDK...\n");
    ret = omega_shutdown();
    if (ret != OMEGA_SUCCESS) {
        print_error("omega_shutdown");
        return 1;
    }
    printf("✓ SDK shutdown successfully\n");

    return 0;
}
