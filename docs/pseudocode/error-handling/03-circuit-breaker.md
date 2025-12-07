# Circuit Breaker Pattern - Pseudocode

## Overview
Advanced circuit breaker implementation for service resilience with state management, health checks, and automatic recovery.

---

## Data Structures

```
DATA STRUCTURE: CircuitBreakerConfig
    failureThreshold: integer (default: 5)
    successThreshold: integer (default: 2)
    timeout: milliseconds (default: 60000)
    halfOpenMaxAttempts: integer (default: 1)
    monitoringWindow: milliseconds (default: 60000)
    errorRateThreshold: float (default: 0.5)  // 50% error rate
    volumeThreshold: integer (default: 10)  // Minimum requests to trip

DATA STRUCTURE: CircuitBreakerState
    state: CLOSED | OPEN | HALF_OPEN
    failureCount: integer
    successCount: integer
    requestCount: integer
    errorCount: integer
    lastFailureTime: timestamp
    lastSuccessTime: timestamp
    lastStateChange: timestamp
    consecutiveSuccesses: integer
    consecutiveFailures: integer
    metrics: CircuitMetrics

DATA STRUCTURE: CircuitMetrics
    totalRequests: integer
    successfulRequests: integer
    failedRequests: integer
    rejectedRequests: integer
    averageResponseTime: milliseconds
    errorRate: float
    lastResetTime: timestamp
    windowStart: timestamp

DATA STRUCTURE: HealthCheckConfig
    enabled: boolean
    interval: milliseconds
    timeout: milliseconds
    healthCheckFunction: function
    successCriteria: function
```

---

## Class: CircuitBreaker

```
CLASS CircuitBreaker:
    config: CircuitBreakerConfig
    state: CircuitBreakerState
    healthCheck: HealthCheckConfig
    listeners: array of EventListener
    metricsWindow: RollingWindow

    CONSTRUCTOR(config: CircuitBreakerConfig):
        this.config ← config
        this.state ← InitializeState()
        this.healthCheck ← InitializeHealthCheck()
        this.listeners ← []
        this.metricsWindow ← RollingWindow(config.monitoringWindow)
        StartMetricsCollector()
        StartHealthCheckMonitor()

    // Main execution method
    METHOD execute(operation: function, fallback: function = null):
        RETURN ExecuteWithCircuitBreaker(operation, fallback)

    // State management
    METHOD getState():
        RETURN this.state.state

    METHOD getMetrics():
        RETURN this.state.metrics

    METHOD reset():
        ResetCircuitBreaker()

    METHOD forceOpen():
        TransitionToState(OPEN, "Manual override")

    METHOD forceClose():
        TransitionToState(CLOSED, "Manual override")

    // Event listeners
    METHOD onStateChange(listener: function):
        this.listeners.append({event: "stateChange", callback: listener})

    METHOD onFailure(listener: function):
        this.listeners.append({event: "failure", callback: listener})

    METHOD onSuccess(listener: function):
        this.listeners.append({event: "success", callback: listener})
```

---

## Algorithm: Execute With Circuit Breaker

```
ALGORITHM: ExecuteWithCircuitBreaker
INPUT: operation (function), fallback (function or null)
OUTPUT: result (any)

BEGIN
    startTime ← GetCurrentTime()

    // Check if circuit allows execution
    IF NOT CanExecute() THEN
        this.state.metrics.rejectedRequests ← this.state.metrics.rejectedRequests + 1

        LogCircuitRejection(this.state.state)
        EmitEvent("requestRejected", {
            state: this.state.state,
            timeSinceOpen: GetCurrentTime() - this.state.lastStateChange
        })

        // Try fallback if available
        IF fallback != null THEN
            TRY
                result ← fallback()
                LogFallbackSuccess()
                RETURN result
            CATCH fallbackError:
                LogFallbackFailure(fallbackError)
                THROW CircuitBreakerOpenError(
                    "Circuit breaker is OPEN and fallback failed",
                    fallbackError
                )
            END TRY
        ELSE
            THROW CircuitBreakerOpenError("Circuit breaker is OPEN")
        END IF
    END IF

    // Increment request counter
    this.state.requestCount ← this.state.requestCount + 1
    this.state.metrics.totalRequests ← this.state.metrics.totalRequests + 1

    // Execute operation
    TRY
        result ← operation()
        responseTime ← GetCurrentTime() - startTime

        // Record success
        RecordSuccess(responseTime)

        RETURN result

    CATCH error:
        responseTime ← GetCurrentTime() - startTime

        // Record failure
        RecordFailure(error, responseTime)

        // Try fallback if available
        IF fallback != null THEN
            TRY
                result ← fallback()
                LogFallbackSuccess()
                RETURN result
            CATCH fallbackError:
                LogFallbackFailure(fallbackError)
                RETHROW error
            END TRY
        ELSE
            RETHROW error
        END IF
    END TRY
END
```

---

## Algorithm: Can Execute

```
ALGORITHM: CanExecute
INPUT: none
OUTPUT: canExecute (boolean)

BEGIN
    currentTime ← GetCurrentTime()

    SWITCH this.state.state:
        CASE CLOSED:
            // Circuit is closed, allow all requests
            RETURN true

        CASE OPEN:
            // Check if timeout period has elapsed
            timeSinceOpen ← currentTime - this.state.lastStateChange

            IF timeSinceOpen >= this.config.timeout THEN
                // Transition to half-open
                TransitionToState(HALF_OPEN, "Timeout elapsed")
                RETURN true
            ELSE
                // Still in open state
                RETURN false
            END IF

        CASE HALF_OPEN:
            // Allow limited requests to test service
            // Use atomic counter to prevent race conditions
            IF this.state.requestCount < this.config.halfOpenMaxAttempts THEN
                RETURN true
            ELSE
                RETURN false
            END IF

        DEFAULT:
            // Unknown state, fail safe and reject
            LogWarning("Unknown circuit breaker state: " + this.state.state)
            RETURN false
    END SWITCH
END
```

---

## Algorithm: Record Success

```
ALGORITHM: RecordSuccess
INPUT: responseTime (milliseconds)
OUTPUT: void

BEGIN
    currentTime ← GetCurrentTime()

    // Update success counters
    this.state.successCount ← this.state.successCount + 1
    this.state.consecutiveSuccesses ← this.state.consecutiveSuccesses + 1
    this.state.consecutiveFailures ← 0
    this.state.lastSuccessTime ← currentTime

    // Update metrics
    this.state.metrics.successfulRequests ← this.state.metrics.successfulRequests + 1
    UpdateAverageResponseTime(responseTime)
    RecordMetricInWindow("success", currentTime)

    // Calculate error rate
    UpdateErrorRate()

    // Emit success event
    EmitEvent("success", {
        state: this.state.state,
        consecutiveSuccesses: this.state.consecutiveSuccesses,
        responseTime: responseTime
    })

    // State transition logic
    SWITCH this.state.state:
        CASE HALF_OPEN:
            // Check if enough successes to close circuit
            IF this.state.consecutiveSuccesses >= this.config.successThreshold THEN
                TransitionToState(CLOSED, "Success threshold reached")
            END IF

        CASE CLOSED:
            // Already closed, no state change needed
            // Just reset failure counters
            this.state.failureCount ← 0
            this.state.consecutiveFailures ← 0

        CASE OPEN:
            // Should not happen in normal flow
            LogWarning("Success recorded while circuit is OPEN")
    END SWITCH
END
```

---

## Algorithm: Record Failure

```
ALGORITHM: RecordFailure
INPUT: error (Error), responseTime (milliseconds)
OUTPUT: void

BEGIN
    currentTime ← GetCurrentTime()

    // Update failure counters
    this.state.failureCount ← this.state.failureCount + 1
    this.state.errorCount ← this.state.errorCount + 1
    this.state.consecutiveFailures ← this.state.consecutiveFailures + 1
    this.state.consecutiveSuccesses ← 0
    this.state.lastFailureTime ← currentTime

    // Update metrics
    this.state.metrics.failedRequests ← this.state.metrics.failedRequests + 1
    UpdateAverageResponseTime(responseTime)
    RecordMetricInWindow("failure", currentTime)

    // Calculate error rate
    UpdateErrorRate()

    // Classify error
    errorType ← ClassifyError(error, {})

    // Emit failure event
    EmitEvent("failure", {
        state: this.state.state,
        consecutiveFailures: this.state.consecutiveFailures,
        errorType: errorType,
        responseTime: responseTime
    })

    // State transition logic
    SWITCH this.state.state:
        CASE CLOSED:
            // Check if we should open the circuit
            shouldOpen ← ShouldOpenCircuit()

            IF shouldOpen THEN
                TransitionToState(OPEN, "Failure threshold exceeded")
            END IF

        CASE HALF_OPEN:
            // Any failure in half-open immediately reopens circuit
            TransitionToState(OPEN, "Failed health check in HALF_OPEN")

        CASE OPEN:
            // Already open, just update metrics
            LogDebug("Additional failure while circuit is OPEN")
    END SWITCH
END
```

---

## Algorithm: Should Open Circuit

```
ALGORITHM: ShouldOpenCircuit
INPUT: none
OUTPUT: shouldOpen (boolean)

BEGIN
    // Check if we have enough volume to make a decision
    IF this.state.requestCount < this.config.volumeThreshold THEN
        RETURN false
    END IF

    // Check consecutive failures
    IF this.state.consecutiveFailures >= this.config.failureThreshold THEN
        LogCircuitTrigger("Consecutive failures: " + this.state.consecutiveFailures)
        RETURN true
    END IF

    // Check error rate in monitoring window
    windowMetrics ← this.metricsWindow.getMetrics()

    IF windowMetrics.errorRate >= this.config.errorRateThreshold THEN
        LogCircuitTrigger("Error rate: " + windowMetrics.errorRate)
        RETURN true
    END IF

    RETURN false
END
```

---

## Algorithm: Transition To State

```
ALGORITHM: TransitionToState
INPUT: newState (State), reason (string)
OUTPUT: void

BEGIN
    oldState ← this.state.state
    currentTime ← GetCurrentTime()

    // Prevent redundant transitions
    IF oldState == newState THEN
        RETURN
    END IF

    // Update state
    this.state.state ← newState
    this.state.lastStateChange ← currentTime

    // Reset counters based on new state
    SWITCH newState:
        CASE CLOSED:
            this.state.failureCount ← 0
            this.state.consecutiveFailures ← 0
            this.state.requestCount ← 0

        CASE OPEN:
            this.state.consecutiveSuccesses ← 0
            this.state.requestCount ← 0

        CASE HALF_OPEN:
            this.state.requestCount ← 0
            this.state.consecutiveSuccesses ← 0
            this.state.consecutiveFailures ← 0
    END SWITCH

    // Log transition
    LogStateTransition(oldState, newState, reason)

    // Emit state change event
    EmitEvent("stateChange", {
        from: oldState,
        to: newState,
        reason: reason,
        timestamp: currentTime,
        metrics: this.state.metrics
    })

    // Emit telemetry
    EmitTelemetry("circuit_breaker_state_change", {
        circuit: this.name,
        oldState: oldState,
        newState: newState,
        reason: reason,
        failureCount: this.state.failureCount,
        errorRate: this.state.metrics.errorRate
    })

    // Alert if opening
    IF newState == OPEN THEN
        AlertOperations("Circuit breaker opened", {
            circuit: this.name,
            reason: reason,
            metrics: this.state.metrics,
            recentErrors: GetRecentErrors(10)
        })
    END IF
END
```

---

## Algorithm: Health Check Monitor

```
ALGORITHM: HealthCheckMonitor
INPUT: none (runs as background task)
OUTPUT: void

BEGIN
    IF NOT this.healthCheck.enabled THEN
        RETURN
    END IF

    WHILE true DO
        // Only run health checks when circuit is OPEN
        IF this.state.state == OPEN THEN
            TRY
                // Execute health check with timeout
                healthCheckPromise ← this.healthCheck.healthCheckFunction()
                result ← AwaitWithTimeout(
                    healthCheckPromise,
                    this.healthCheck.timeout
                )

                // Evaluate success criteria
                isHealthy ← this.healthCheck.successCriteria(result)

                IF isHealthy THEN
                    LogHealthCheckSuccess()

                    // Transition to half-open to test service
                    TransitionToState(HALF_OPEN, "Health check passed")
                ELSE
                    LogHealthCheckFailure("Success criteria not met")
                END IF

            CATCH error:
                LogHealthCheckFailure(error.message)
                // Stay in OPEN state
            END TRY
        END IF

        // Wait for next interval
        Sleep(this.healthCheck.interval)
    END WHILE
END
```

---

## Algorithm: Update Error Rate

```
ALGORITHM: UpdateErrorRate
INPUT: none
OUTPUT: void

BEGIN
    totalRequests ← this.state.metrics.totalRequests
    failedRequests ← this.state.metrics.failedRequests

    IF totalRequests > 0 THEN
        this.state.metrics.errorRate ← failedRequests / totalRequests
    ELSE
        this.state.metrics.errorRate ← 0.0
    END IF

    // Also update rolling window error rate
    windowMetrics ← this.metricsWindow.calculate()

    LogDebug("Error rate updated", {
        overall: this.state.metrics.errorRate,
        window: windowMetrics.errorRate,
        totalRequests: totalRequests,
        failedRequests: failedRequests
    })
END
```

---

## Data Structure: Rolling Window

```
CLASS RollingWindow:
    windowSize: milliseconds
    buckets: array of Bucket
    bucketDuration: milliseconds

    CONSTRUCTOR(windowSize: milliseconds):
        this.windowSize ← windowSize
        this.bucketDuration ← windowSize / 10  // 10 buckets
        this.buckets ← []

    METHOD record(eventType: string, timestamp: timestamp):
        bucketIndex ← GetBucketIndex(timestamp)
        bucket ← GetOrCreateBucket(bucketIndex, timestamp)

        IF eventType == "success" THEN
            bucket.successCount ← bucket.successCount + 1
        ELSE IF eventType == "failure" THEN
            bucket.failureCount ← bucket.failureCount + 1
        END IF

        bucket.totalCount ← bucket.totalCount + 1

        // Clean old buckets
        CleanOldBuckets(timestamp)

    METHOD getMetrics():
        currentTime ← GetCurrentTime()
        CleanOldBuckets(currentTime)

        totalSuccess ← 0
        totalFailure ← 0

        FOR EACH bucket IN this.buckets DO
            totalSuccess ← totalSuccess + bucket.successCount
            totalFailure ← totalFailure + bucket.failureCount
        END FOR

        totalRequests ← totalSuccess + totalFailure
        errorRate ← 0.0

        IF totalRequests > 0 THEN
            errorRate ← totalFailure / totalRequests
        END IF

        RETURN {
            totalRequests: totalRequests,
            successCount: totalSuccess,
            failureCount: totalFailure,
            errorRate: errorRate
        }
```

---

## Complexity Analysis

**ExecuteWithCircuitBreaker:**
- Time Complexity: O(1) + O(operation) - constant overhead plus operation time
- Space Complexity: O(1) - fixed state storage

**State Transitions:**
- Time Complexity: O(1) - direct state updates
- Space Complexity: O(1)

**Rolling Window Metrics:**
- Time Complexity: O(b) where b = number of buckets (typically 10)
- Space Complexity: O(b)

---

## Design Patterns

1. **State Pattern**: Circuit breaker states (closed, open, half-open)
2. **Observer Pattern**: Event listeners for state changes and failures
3. **Strategy Pattern**: Configurable health check and success criteria
4. **Sliding Window**: Rolling metrics calculation for error rate
