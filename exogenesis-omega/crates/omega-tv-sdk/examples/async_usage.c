/**
 * @file async_usage.c
 * @brief Example of using Omega SDK with background sync thread
 *
 * This demonstrates how to use the SDK in a multi-threaded environment
 * with periodic background synchronization.
 *
 * Compile:
 *   gcc async_usage.c -L../target/release -lomega_tv_sdk -pthread -o async_usage
 */

#include "../include/omega_sdk.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>
#include <signal.h>

#define BUFFER_SIZE 8192
#define SYNC_INTERVAL_SECONDS 600 // 10 minutes

static volatile int running = 1;

void signal_handler(int signum) {
    printf("\nReceived signal %d, shutting down...\n", signum);
    running = 0;
}

void* sync_thread(void* arg) {
    printf("Sync thread started (syncing every %d seconds)\n", SYNC_INTERVAL_SECONDS);

    while (running) {
        // Wait for sync interval
        for (int i = 0; i < SYNC_INTERVAL_SECONDS && running; i++) {
            sleep(1);
        }

        if (!running) break;

        printf("⚡ Background sync starting...\n");
        int ret = omega_sync();

        if (ret == OMEGA_SUCCESS) {
            printf("✓ Background sync completed\n");
        } else {
            printf("⚠ Background sync failed: %s\n", omega_get_last_error());
        }
    }

    printf("Sync thread exiting\n");
    return NULL;
}

int main() {
    pthread_t sync_tid;
    char buffer[BUFFER_SIZE];

    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);

    printf("Omega TV SDK - Async Example\n");
    printf("============================\n\n");

    // Initialize SDK
    printf("Initializing SDK...\n");
    int ret = omega_init("/tmp/omega_async", "http://localhost:8080");
    if (ret != OMEGA_SUCCESS) {
        fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
        return 1;
    }
    printf("✓ SDK initialized\n\n");

    // Start background sync thread
    if (pthread_create(&sync_tid, NULL, sync_thread, NULL) != 0) {
        fprintf(stderr, "Failed to create sync thread\n");
        omega_shutdown();
        return 1;
    }

    // Simulate TV operation
    printf("Simulating TV operation (Press Ctrl+C to exit)\n\n");

    int event_count = 0;
    while (running) {
        // Simulate getting recommendations every 5 seconds
        const char* context = "{"
            "\"genre\":\"action\","
            "\"time\":\"evening\""
        "}";

        ret = omega_recommend(context, buffer, BUFFER_SIZE);
        if (ret == OMEGA_SUCCESS) {
            printf("📺 Recommendations served (#%d)\n", ++event_count);
        } else {
            fprintf(stderr, "Recommend failed: %s\n", omega_get_last_error());
        }

        // Simulate viewing event
        const char* event = "{"
            "\"content_id\":\"movie123\","
            "\"watch_percentage\":0.75"
        "}";

        ret = omega_observe(event);
        if (ret == OMEGA_SUCCESS) {
            printf("👁  Viewing event recorded\n");
        }

        printf("\n");
        sleep(5);
    }

    // Wait for sync thread to exit
    printf("Waiting for sync thread...\n");
    pthread_join(sync_tid, NULL);

    // Shutdown
    printf("Shutting down SDK...\n");
    omega_shutdown();
    printf("✓ Cleanup complete\n");

    return 0;
}
