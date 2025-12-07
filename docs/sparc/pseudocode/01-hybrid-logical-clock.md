# Hybrid Logical Clock (HLC) Pseudocode

## Overview
Provides causality tracking and total ordering for distributed events with minimal clock drift impact.

## Data Structures

```
STRUCTURE HLC:
    physical_time: uint48      // milliseconds since epoch (281 trillion years)
    logical_counter: uint16    // logical tick counter (0-65535)

CONSTANTS:
    MAX_LOGICAL_COUNTER = 65535
    CLOCK_DRIFT_THRESHOLD = 5000  // 5 seconds
```

## Core Algorithms

### 1. HLC Creation and Increment

```
ALGORITHM: CreateHLC
INPUT: wall_clock (milliseconds)
OUTPUT: hlc (HLC)

BEGIN
    hlc.physical_time ← wall_clock
    hlc.logical_counter ← 0
    RETURN hlc
END

ALGORITHM: IncrementHLC
INPUT: current_hlc (HLC), wall_clock (uint48)
OUTPUT: new_hlc (HLC)

BEGIN
    new_hlc ← HLC()

    // Update physical time to maximum of current and wall clock
    new_hlc.physical_time ← MAX(current_hlc.physical_time, wall_clock)

    // Increment logical counter if physical time unchanged
    IF new_hlc.physical_time == current_hlc.physical_time THEN
        IF current_hlc.logical_counter >= MAX_LOGICAL_COUNTER THEN
            // Counter overflow - force physical time advancement
            new_hlc.physical_time ← new_hlc.physical_time + 1
            new_hlc.logical_counter ← 0
        ELSE
            new_hlc.logical_counter ← current_hlc.logical_counter + 1
        END IF
    ELSE
        // Physical time advanced, reset logical counter
        new_hlc.logical_counter ← 0
    END IF

    RETURN new_hlc
END
```

### 2. HLC Receive and Merge

```
ALGORITHM: ReceiveHLC
INPUT: local_hlc (HLC), remote_hlc (HLC), wall_clock (uint48)
OUTPUT: merged_hlc (HLC)

BEGIN
    merged_hlc ← HLC()

    // Calculate maximum physical time
    max_physical ← MAX(local_hlc.physical_time, remote_hlc.physical_time, wall_clock)
    merged_hlc.physical_time ← max_physical

    // Determine logical counter based on physical time equality
    IF max_physical == local_hlc.physical_time AND max_physical == remote_hlc.physical_time THEN
        // Both clocks at same physical time - use max logical + 1
        merged_hlc.logical_counter ← MAX(local_hlc.logical_counter, remote_hlc.logical_counter) + 1

    ELSE IF max_physical == local_hlc.physical_time THEN
        // Local clock ahead - increment local logical
        merged_hlc.logical_counter ← local_hlc.logical_counter + 1

    ELSE IF max_physical == remote_hlc.physical_time THEN
        // Remote clock ahead - use remote logical
        merged_hlc.logical_counter ← remote_hlc.logical_counter

    ELSE
        // Wall clock ahead - reset logical
        merged_hlc.logical_counter ← 0
    END IF

    // Handle counter overflow
    IF merged_hlc.logical_counter > MAX_LOGICAL_COUNTER THEN
        merged_hlc.physical_time ← merged_hlc.physical_time + 1
        merged_hlc.logical_counter ← 0
    END IF

    // Detect excessive clock drift
    IF ABS(wall_clock - merged_hlc.physical_time) > CLOCK_DRIFT_THRESHOLD THEN
        LogWarning("Clock drift detected", wall_clock, merged_hlc.physical_time)
    END IF

    RETURN merged_hlc
END
```

### 3. HLC Comparison (Total Ordering)

```
ALGORITHM: CompareHLC
INPUT: hlc_a (HLC), hlc_b (HLC)
OUTPUT: comparison (-1 | 0 | 1)

BEGIN
    // Compare physical time first
    IF hlc_a.physical_time < hlc_b.physical_time THEN
        RETURN -1
    ELSE IF hlc_a.physical_time > hlc_b.physical_time THEN
        RETURN 1
    END IF

    // Physical times equal - compare logical counters
    IF hlc_a.logical_counter < hlc_b.logical_counter THEN
        RETURN -1
    ELSE IF hlc_a.logical_counter > hlc_b.logical_counter THEN
        RETURN 1
    ELSE
        RETURN 0
    END IF
END

ALGORITHM: HLCLessThan
INPUT: hlc_a (HLC), hlc_b (HLC)
OUTPUT: boolean

BEGIN
    RETURN CompareHLC(hlc_a, hlc_b) < 0
END

ALGORITHM: HLCEquals
INPUT: hlc_a (HLC), hlc_b (HLC)
OUTPUT: boolean

BEGIN
    RETURN CompareHLC(hlc_a, hlc_b) == 0
END
```

### 4. HLC Serialization

```
ALGORITHM: SerializeHLC
INPUT: hlc (HLC)
OUTPUT: timestamp_string (string)

BEGIN
    // Format: "physical_time-logical_counter"
    // Example: "1701234567890-42"
    timestamp_string ← CONCAT(
        ToString(hlc.physical_time),
        "-",
        ToString(hlc.logical_counter)
    )
    RETURN timestamp_string
END

ALGORITHM: DeserializeHLC
INPUT: timestamp_string (string)
OUTPUT: hlc (HLC) or error

BEGIN
    parts ← Split(timestamp_string, "-")

    IF Length(parts) != 2 THEN
        RETURN error("Invalid HLC format")
    END IF

    physical ← ParseUInt48(parts[0])
    logical ← ParseUInt16(parts[1])

    IF physical is null OR logical is null THEN
        RETURN error("Invalid HLC values")
    END IF

    hlc ← HLC()
    hlc.physical_time ← physical
    hlc.logical_counter ← logical

    RETURN hlc
END
```

### 5. HLC Encoding (64-bit Optimization)

```
ALGORITHM: EncodeHLC
INPUT: hlc (HLC)
OUTPUT: encoded (uint64)

BEGIN
    // Pack into 64 bits: [48 bits physical][16 bits logical]
    encoded ← (hlc.physical_time << 16) | hlc.logical_counter
    RETURN encoded
END

ALGORITHM: DecodeHLC
INPUT: encoded (uint64)
OUTPUT: hlc (HLC)

BEGIN
    hlc ← HLC()
    hlc.physical_time ← (encoded >> 16) & 0xFFFFFFFFFFFF
    hlc.logical_counter ← encoded & 0xFFFF
    RETURN hlc
END
```

## Complexity Analysis

### Time Complexity
- `CreateHLC`: O(1)
- `IncrementHLC`: O(1)
- `ReceiveHLC`: O(1)
- `CompareHLC`: O(1)
- `SerializeHLC`: O(1)
- `DeserializeHLC`: O(1)
- `EncodeHLC`: O(1)
- `DecodeHLC`: O(1)

### Space Complexity
- HLC structure: O(1) - 8 bytes (64 bits)
- Serialized form: O(1) - ~25 bytes max (string)
- All operations: O(1) auxiliary space

## Properties

### Causality Preservation
- If event A happens-before event B, then HLC(A) < HLC(B)
- Monotonically increasing per device
- Respects causal ordering across devices

### Total Ordering
- Any two HLCs can be compared
- Comparison is transitive: if A < B and B < C, then A < C
- Tie-breaking via logical counter ensures deterministic ordering

### Clock Drift Tolerance
- Physical time bounds logical time drift
- Logical counter prevents time reversal
- Graceful handling of synchronized clocks

## Usage Example

```
// Device A sends event
local_hlc ← IncrementHLC(device_a_clock, GetWallClock())
SendMessage(event_data, SerializeHLC(local_hlc))

// Device B receives event
remote_hlc ← DeserializeHLC(message.timestamp)
device_b_clock ← ReceiveHLC(device_b_clock, remote_hlc, GetWallClock())

// Determine event ordering
IF HLCLessThan(event1.hlc, event2.hlc) THEN
    // event1 happened before event2
    ProcessEventsInOrder(event1, event2)
END IF
```

## Performance Characteristics

- **Latency**: <1μs for increment/compare operations
- **Memory**: 8 bytes per timestamp
- **Network**: 25 bytes serialized (vs 8 bytes for simple timestamp)
- **Accuracy**: Nanosecond precision possible (implementation dependent)
- **Scalability**: Unlimited devices, no coordination required

## Edge Cases

1. **Counter Overflow**: Forces physical time increment
2. **Clock Skew**: Detected and logged when >5 seconds
3. **Time Reversal**: Prevented by MAX operation
4. **Concurrent Events**: Disambiguated by logical counter
5. **Network Partition**: Causality preserved per partition, merged on rejoin
