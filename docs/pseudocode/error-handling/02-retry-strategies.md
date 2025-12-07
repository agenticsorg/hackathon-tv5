# Retry Strategy Engine - Pseudocode

## Overview
Comprehensive retry mechanism with exponential backoff, jitter, circuit breakers, and fallback strategies.

---

## Data Structures

```
DATA STRUCTURE: RetryConfig
    strategy: EXPONENTIAL | LINEAR | IMMEDIATE | CIRCUIT_BREAKER
    baseDelay: milliseconds
    maxDelay: milliseconds
    maxAttempts: integer
    jitter: boolean
    backoffMultiplier: float (default: 2.0)
    retryableStatusCodes: array of integers
    retryableErrorTypes: array of ErrorType

DATA STRUCTURE: RetryState
    attemptCount: integer
    lastAttemptTime: timestamp
    totalDelay: milliseconds
    errors: array of Error
    circuitState: CLOSED | OPEN | HALF_OPEN (if using circuit breaker)
    successCount: integer (for circuit breaker)
    failureCount: integer (for circuit breaker)

DATA STRUCTURE: RetryResult
    success: boolean
    result: any
    error: Error or null
    attemptsMade: integer
    totalTime: milliseconds
    strategy: string
```

---

## Algorithm: Retry With Strategy

```
ALGORITHM: RetryWithStrategy
INPUT: operation (function), config (RetryConfig), context (object)
OUTPUT: result (RetryResult)

BEGIN
    state ← InitializeRetryState()
    startTime ← GetCurrentTime()

    FOR attempt FROM 1 TO config.maxAttempts DO
        state.attemptCount ← attempt

        // Check circuit breaker state
        IF config.strategy == CIRCUIT_BREAKER THEN
            IF NOT CanAttempt(state) THEN
                RETURN RetryResult(
                    success: false,
                    error: CircuitBreakerOpenError,
                    attemptsMade: attempt - 1,
                    totalTime: GetCurrentTime() - startTime,
                    strategy: "CIRCUIT_BREAKER"
                )
            END IF
        END IF

        TRY
            // Log attempt
            LogRetryAttempt(attempt, config.maxAttempts, context)

            // Execute operation
            result ← operation()

            // Record success
            RecordSuccess(state, config)

            // Return successful result
            RETURN RetryResult(
                success: true,
                result: result,
                error: null,
                attemptsMade: attempt,
                totalTime: GetCurrentTime() - startTime,
                strategy: config.strategy
            )

        CATCH error:
            // Classify error
            errorType ← ClassifyError(error, context)

            // Record failure
            RecordFailure(state, config, error)
            state.errors.append(error)

            // Check if error is retryable
            IF NOT IsRetryable(error, errorType, config) THEN
                LogNonRetryableError(error, errorType)
                RETURN RetryResult(
                    success: false,
                    error: error,
                    attemptsMade: attempt,
                    totalTime: GetCurrentTime() - startTime,
                    strategy: config.strategy
                )
            END IF

            // Check if we have attempts left
            IF attempt >= config.maxAttempts THEN
                LogMaxAttemptsReached(error, attempt)
                RETURN RetryResult(
                    success: false,
                    error: MaxRetriesExceededError(state.errors),
                    attemptsMade: attempt,
                    totalTime: GetCurrentTime() - startTime,
                    strategy: config.strategy
                )
            END IF

            // Calculate delay before next attempt
            delay ← CalculateDelay(attempt, config, state)
            state.totalDelay ← state.totalDelay + delay

            // Log retry delay
            LogRetryDelay(attempt, delay, error)

            // Wait before retry
            Sleep(delay)
            state.lastAttemptTime ← GetCurrentTime()
        END TRY
    END FOR

    // Should never reach here, but handle as failure
    RETURN RetryResult(
        success: false,
        error: UnexpectedRetryError,
        attemptsMade: config.maxAttempts,
        totalTime: GetCurrentTime() - startTime,
        strategy: config.strategy
    )
END
```

---

## Algorithm: Calculate Delay

```
ALGORITHM: CalculateDelay
INPUT: attempt (integer), config (RetryConfig), state (RetryState)
OUTPUT: delay (milliseconds)

BEGIN
    baseDelay ← config.baseDelay

    SWITCH config.strategy:
        CASE EXPONENTIAL:
            // Exponential backoff: delay = baseDelay * (multiplier ^ (attempt - 1))
            delay ← baseDelay * (config.backoffMultiplier ^ (attempt - 1))

        CASE LINEAR:
            // Linear backoff: delay = baseDelay * attempt
            delay ← baseDelay * attempt

        CASE IMMEDIATE:
            // No delay
            delay ← 0

        CASE CIRCUIT_BREAKER:
            // Use exponential for circuit breaker
            delay ← baseDelay * (config.backoffMultiplier ^ (attempt - 1))

        DEFAULT:
            delay ← baseDelay
    END SWITCH

    // Apply maximum delay cap
    IF delay > config.maxDelay THEN
        delay ← config.maxDelay
    END IF

    // Add jitter to prevent thundering herd
    IF config.jitter == true THEN
        jitterAmount ← Random(0, delay * 0.3)  // 0-30% jitter
        delay ← delay + jitterAmount
    END IF

    RETURN delay
END
```

---

## Algorithm: Is Retryable

```
ALGORITHM: IsRetryable
INPUT: error (Error), errorType (ErrorType), config (RetryConfig)
OUTPUT: retryable (boolean)

BEGIN
    // Check if error type is in retryable list
    IF errorType IN config.retryableErrorTypes THEN
        RETURN true
    END IF

    // Check if status code is retryable
    IF error.statusCode EXISTS THEN
        IF error.statusCode IN config.retryableStatusCodes THEN
            RETURN true
        END IF
    END IF

    // Check error metadata
    metadata ← GetErrorMetadata(errorType)
    IF metadata.retryable == true THEN
        RETURN true
    END IF

    // Default: not retryable
    RETURN false
END
```

---

## Algorithm: Circuit Breaker - Can Attempt

```
ALGORITHM: CanAttempt
INPUT: state (RetryState)
OUTPUT: canAttempt (boolean)

CONSTANTS:
    FAILURE_THRESHOLD = 5
    SUCCESS_THRESHOLD = 2
    OPEN_TIMEOUT = 60000ms  // 60 seconds
    HALF_OPEN_MAX_ATTEMPTS = 1

BEGIN
    currentTime ← GetCurrentTime()

    SWITCH state.circuitState:
        CASE CLOSED:
            // Circuit is closed, allow attempts
            RETURN true

        CASE OPEN:
            // Check if timeout has elapsed
            timeSinceLastFailure ← currentTime - state.lastAttemptTime

            IF timeSinceLastFailure >= OPEN_TIMEOUT THEN
                // Transition to half-open
                state.circuitState ← HALF_OPEN
                state.successCount ← 0
                state.failureCount ← 0
                LogCircuitStateChange("OPEN", "HALF_OPEN")
                RETURN true
            ELSE
                // Still in timeout period
                LogCircuitBreakerBlocked(timeSinceLastFailure)
                RETURN false
            END IF

        CASE HALF_OPEN:
            // Allow limited attempts to test service
            IF state.attemptCount < HALF_OPEN_MAX_ATTEMPTS THEN
                RETURN true
            ELSE
                RETURN false
            END IF

        DEFAULT:
            RETURN true
    END SWITCH
END
```

---

## Algorithm: Record Success

```
ALGORITHM: RecordSuccess
INPUT: state (RetryState), config (RetryConfig)
OUTPUT: void

CONSTANTS:
    SUCCESS_THRESHOLD = 2

BEGIN
    IF config.strategy == CIRCUIT_BREAKER THEN
        state.successCount ← state.successCount + 1
        state.failureCount ← 0  // Reset failure count

        SWITCH state.circuitState:
            CASE HALF_OPEN:
                // Check if we have enough successes to close circuit
                IF state.successCount >= SUCCESS_THRESHOLD THEN
                    state.circuitState ← CLOSED
                    state.failureCount ← 0
                    state.successCount ← 0
                    LogCircuitStateChange("HALF_OPEN", "CLOSED")
                    EmitTelemetry("circuit_breaker_closed", {
                        successCount: state.successCount
                    })
                END IF

            CASE CLOSED:
                // Already closed, just track metrics
                EmitTelemetry("operation_success", {
                    attemptCount: state.attemptCount
                })

            CASE OPEN:
                // Should not happen, but log anomaly
                LogWarning("Success recorded in OPEN state")
        END SWITCH
    END IF

    // Log success metrics
    EmitTelemetry("retry_success", {
        attemptCount: state.attemptCount,
        totalDelay: state.totalDelay,
        strategy: config.strategy
    })
END
```

---

## Algorithm: Record Failure

```
ALGORITHM: RecordFailure
INPUT: state (RetryState), config (RetryConfig), error (Error)
OUTPUT: void

CONSTANTS:
    FAILURE_THRESHOLD = 5

BEGIN
    IF config.strategy == CIRCUIT_BREAKER THEN
        state.failureCount ← state.failureCount + 1
        state.successCount ← 0  // Reset success count

        SWITCH state.circuitState:
            CASE CLOSED:
                // Check if we've exceeded failure threshold
                IF state.failureCount >= FAILURE_THRESHOLD THEN
                    state.circuitState ← OPEN
                    state.lastAttemptTime ← GetCurrentTime()
                    LogCircuitStateChange("CLOSED", "OPEN")
                    EmitTelemetry("circuit_breaker_opened", {
                        failureCount: state.failureCount,
                        errors: state.errors
                    })
                    AlertOperations("Circuit breaker opened", {
                        failureCount: state.failureCount,
                        lastError: error
                    })
                END IF

            CASE HALF_OPEN:
                // Any failure in half-open state reopens circuit
                state.circuitState ← OPEN
                state.lastAttemptTime ← GetCurrentTime()
                LogCircuitStateChange("HALF_OPEN", "OPEN")
                EmitTelemetry("circuit_breaker_reopened", {
                    error: error
                })

            CASE OPEN:
                // Already open, just log
                LogDebug("Failure recorded in OPEN state")
        END SWITCH
    END IF

    // Log failure metrics
    EmitTelemetry("retry_failure", {
        attemptCount: state.attemptCount,
        errorType: ClassifyError(error),
        strategy: config.strategy
    })
END
```

---

## Predefined Retry Configurations

```
CONFIGURATION: DefaultRetryConfigs

NetworkTimeout:
    strategy: EXPONENTIAL
    baseDelay: 1000ms
    maxDelay: 30000ms
    maxAttempts: 5
    jitter: true
    backoffMultiplier: 2.0
    retryableStatusCodes: []
    retryableErrorTypes: [NetworkError.ConnectionTimeout]

APIRateLimit:
    strategy: LINEAR
    baseDelay: 2000ms
    maxDelay: 60000ms
    maxAttempts: 3
    jitter: true
    backoffMultiplier: 1.5
    retryableStatusCodes: [429]
    retryableErrorTypes: [APIError.RateLimitExceeded]

ServiceUnavailable:
    strategy: CIRCUIT_BREAKER
    baseDelay: 5000ms
    maxDelay: 60000ms
    maxAttempts: 3
    jitter: true
    backoffMultiplier: 2.0
    retryableStatusCodes: [502, 503, 504]
    retryableErrorTypes: [APIError.ServiceUnavailable]

AuthTokenRefresh:
    strategy: IMMEDIATE
    baseDelay: 0ms
    maxDelay: 0ms
    maxAttempts: 1
    jitter: false
    backoffMultiplier: 1.0
    retryableStatusCodes: [401]
    retryableErrorTypes: [AuthenticationError.TokenExpired]
```

---

## Complexity Analysis

**RetryWithStrategy Algorithm:**
- Time Complexity: O(n * m) where n = maxAttempts, m = operation time
- Space Complexity: O(n) for storing error history

**CalculateDelay Algorithm:**
- Time Complexity: O(1) - simple arithmetic operations
- Space Complexity: O(1)

**Circuit Breaker Operations:**
- Time Complexity: O(1) - state checks and transitions
- Space Complexity: O(1) - fixed state storage

---

## Design Patterns

1. **Strategy Pattern**: Different retry strategies (exponential, linear, circuit breaker)
2. **State Pattern**: Circuit breaker state transitions (closed, open, half-open)
3. **Template Method**: Retry logic template with strategy-specific delay calculation
4. **Observer Pattern**: Telemetry and logging hooks throughout retry flow
