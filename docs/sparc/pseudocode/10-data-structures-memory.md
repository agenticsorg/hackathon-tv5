# Data Structures and Memory Layout

## Overview
Comprehensive documentation of all data structures, memory layouts, and storage strategies for the Real-Time Synchronization system.

## Core Data Structures

### 1. Hybrid Logical Clock (HLC)

```
MEMORY LAYOUT (64 bits):
┌─────────────────────────────────────────────────┬─────────────────┐
│         Physical Time (48 bits)                 │  Logical (16)   │
│         0xFFFFFFFFFFFF                          │    0xFFFF       │
└─────────────────────────────────────────────────┴─────────────────┘

Bit positions: [63...16][15...0]

Fields:
  physical_time: uint48  // Milliseconds since epoch (Jan 1, 1970)
  logical_counter: uint16 // Logical tick (0-65535)

Size: 8 bytes (stack-allocated)
Alignment: 8-byte boundary
Endianness: Little-endian

Serialized formats:
  String: "1701234567890-42" (~20-25 bytes)
  Binary: 8 bytes (direct encoding)
  JSON: {"pt":1701234567890,"lc":42} (~30 bytes)

Memory efficiency: 100% (no padding)
```

### 2. LWW-Register

```
STRUCTURE Layout:
┌────────────────────────────────────────────────────────┐
│ value: T                    (size varies)              │
├────────────────────────────────────────────────────────┤
│ timestamp: HLC              (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ device_id: string           (16-64 bytes typical)      │
└────────────────────────────────────────────────────────┘

Size calculation:
  Base overhead: 8 bytes (HLC) + 24 bytes (device_id avg) = 32 bytes
  Total: 32 + sizeof(value)

Examples:
  WatchProgress: 32 + 48 = 80 bytes
    - media_id: 16 bytes
    - position: 8 bytes (float64)
    - duration: 8 bytes (float64)
    - completed: 1 byte (bool)
    - last_watched: 8 bytes (timestamp)
    - padding: 7 bytes

  UserPreference (subtitle): 32 + 8 = 40 bytes
    - value: 8 bytes (language code)

Memory alignment: 8-byte aligned for performance
Cache efficiency: Single cache line (64 bytes) for most values
```

### 3. OR-Set

```
STRUCTURE Layout:
┌─────────────────────────────────────────────────────────┐
│ elements: Map<T, Set<UniqueTag>>                        │
│   - Hash map overhead: 24 bytes                         │
│   - Bucket array: 16 bytes per bucket                   │
│   - Entry: 8 bytes (key) + 8 bytes (value ptr)          │
├─────────────────────────────────────────────────────────┤
│ tombstones: Map<T, Set<UniqueTag>>                      │
│   - Same structure as elements                          │
└─────────────────────────────────────────────────────────┘

UniqueTag layout (32 bytes):
┌────────────────────────────────────────────────────────┐
│ device_id: string           (16 bytes avg)            │
├────────────────────────────────────────────────────────┤
│ hlc: HLC                    (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ sequence: uint32            (4 bytes)                  │
├────────────────────────────────────────────────────────┤
│ padding                     (4 bytes)                  │
└────────────────────────────────────────────────────────┘

Size calculation for watchlist:
  Base maps: 2 × 24 = 48 bytes
  Element overhead per item: 16 bytes (map entry)
  Tag set per item: 24 bytes (set overhead)
  Tags per item (avg 2): 2 × 32 = 64 bytes
  Per-item total: 16 + 24 + 64 = 104 bytes

  100-item watchlist: 48 + (100 × 104) = 10,448 bytes (~10KB)

Memory optimization strategies:
  1. Interned device IDs (reduce from 16 to 4 bytes)
  2. Tag pooling (reuse allocated tags)
  3. Bloom filter for tombstones (faster membership test)
  4. Compressed sets for inactive items
```

### 4. Device Presence

```
STRUCTURE Layout (200 bytes):
┌────────────────────────────────────────────────────────┐
│ device_id: string           (16 bytes)                 │
├────────────────────────────────────────────────────────┤
│ user_id: string             (16 bytes)                 │
├────────────────────────────────────────────────────────┤
│ device_type: enum           (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ device_name: string         (32 bytes)                 │
├────────────────────────────────────────────────────────┤
│ status: enum                (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ last_heartbeat: timestamp   (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ last_activity: timestamp    (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ current_media_id: string?   (16 bytes + 1 null flag)   │
├────────────────────────────────────────────────────────┤
│ playback_position: float?   (8 bytes + 1 null flag)    │
├────────────────────────────────────────────────────────┤
│ network_quality: enum       (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ battery_level: uint8?       (1 byte + 1 null flag)     │
├────────────────────────────────────────────────────────┤
│ capabilities: Set<enum>     (16 bytes for bitset)      │
├────────────────────────────────────────────────────────┤
│ padding                     (66 bytes)                 │
└────────────────────────────────────────────────────────┘

Capability bitset encoding:
  Bit 0: PLAYBACK
  Bit 1: REMOTE_CONTROL
  Bit 2: OFFLINE_DOWNLOAD
  Bit 3: HDR
  Bit 4: 4K
  Bit 5: DOLBY_ATMOS
  Bits 6-127: Reserved

Memory per user (5 devices): 5 × 200 = 1,000 bytes
```

### 5. PubNub Message Envelope

```
STRUCTURE Layout:
┌────────────────────────────────────────────────────────┐
│ channel: string             (32 bytes avg)             │
├────────────────────────────────────────────────────────┤
│ message_type: string        (16 bytes avg)             │
├────────────────────────────────────────────────────────┤
│ payload: any                (variable)                 │
├────────────────────────────────────────────────────────┤
│ timestamp: HLC              (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ sender_device_id: string    (16 bytes)                 │
├────────────────────────────────────────────────────────┤
│ message_id: UUID            (16 bytes)                 │
└────────────────────────────────────────────────────────┘

Base size: 88 bytes + payload
Typical payloads:
  - Watch progress: 48 bytes
  - Watchlist add: 32 bytes
  - Heartbeat: 64 bytes
  - Remote command: 200 bytes

Serialized (JSON):
  Overhead: ~150 bytes
  Total: 150 + payload

Serialized (Binary with schema):
  Overhead: ~50 bytes
  Total: 50 + payload
  Savings: 66% on overhead
```

### 6. Local Change Queue

```
STRUCTURE LocalChange (200 bytes):
┌────────────────────────────────────────────────────────┐
│ change_id: UUID             (16 bytes)                 │
├────────────────────────────────────────────────────────┤
│ change_type: enum           (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ resource_type: enum         (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ resource_id: string         (16 bytes)                 │
├────────────────────────────────────────────────────────┤
│ operation: CRDTOperation    (variable, ~100 bytes avg) │
├────────────────────────────────────────────────────────┤
│ timestamp: HLC              (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ retry_count: uint8          (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ status: enum                (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ padding                     (56 bytes)                 │
└────────────────────────────────────────────────────────┘

Queue implementation:
  Type: Circular buffer (ring buffer)
  Capacity: 1000 entries
  Total size: 1000 × 200 = 200KB

Queue operations:
  enqueue: O(1) - write to tail, advance pointer
  dequeue: O(1) - read from head, advance pointer
  peek: O(1) - read head without advancing

Memory efficiency: 100% utilization (no linked list overhead)
Cache efficiency: Sequential access pattern
```

### 7. LRU Cache

```
STRUCTURE Layout:
┌────────────────────────────────────────────────────────┐
│ map: HashMap<string, Node*> (24 bytes + entries)       │
├────────────────────────────────────────────────────────┤
│ head: Node*                 (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ tail: Node*                 (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ capacity: integer           (4 bytes)                  │
├────────────────────────────────────────────────────────┤
│ size: integer               (4 bytes)                  │
└────────────────────────────────────────────────────────┘

Node structure (48 bytes):
┌────────────────────────────────────────────────────────┐
│ key: string                 (16 bytes avg)             │
├────────────────────────────────────────────────────────┤
│ value: any                  (variable)                 │
├────────────────────────────────────────────────────────┤
│ prev: Node*                 (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ next: Node*                 (8 bytes)                  │
└────────────────────────────────────────────────────────┘

Capacity 1000:
  Base: 48 bytes
  Map overhead: 24 bytes
  Entries: 1000 × (16 + 8) = 24KB (map entries)
  Nodes: 1000 × (48 + avg_value_size)

  Typical (50 byte values): 48 + 24 + 24,000 + (1000 × 98) = 122KB

Memory vs performance tradeoff:
  Small cache (100): Low memory, high miss rate
  Medium cache (1000): Balanced
  Large cache (5000): High memory, low miss rate
```

### 8. Remote Command State

```
STRUCTURE Layout (400 bytes):
┌────────────────────────────────────────────────────────┐
│ command: RemoteCommand      (300 bytes)                │
│   ├─ command_id: UUID       (16 bytes)                 │
│   ├─ source_device_id       (16 bytes)                 │
│   ├─ target_device_id       (16 bytes)                 │
│   ├─ command_type: enum     (1 byte)                   │
│   ├─ parameters: Map        (200 bytes avg)            │
│   ├─ timestamp: HLC         (8 bytes)                  │
│   ├─ timeout_ms: uint32     (4 bytes)                  │
│   ├─ requires_ack: bool     (1 byte)                   │
│   └─ padding                (38 bytes)                 │
├────────────────────────────────────────────────────────┤
│ status: enum                (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ sent_at: timestamp          (8 bytes)                  │
├────────────────────────────────────────────────────────┤
│ ack_received_at: timestamp? (8 bytes + 1 null)         │
├────────────────────────────────────────────────────────┤
│ retry_count: uint8          (1 byte)                   │
├────────────────────────────────────────────────────────┤
│ timeout_timer_id: string?   (16 bytes + 1 null)        │
├────────────────────────────────────────────────────────┤
│ padding                     (64 bytes)                 │
└────────────────────────────────────────────────────────┘

Max pending commands: 10
Total memory: 10 × 400 = 4KB
```

## Memory Management Strategies

### 1. Object Pooling

```
ALGORITHM: CreateObjectPool
INPUT: object_type (Type), pool_size (integer)
OUTPUT: pool (ObjectPool)

Pool benefits:
  - Reduces GC pressure by 70-90%
  - Improves allocation latency from ~1ms to ~10μs
  - Better cache locality

Pool for UniqueTag (high churn):
  Pool size: 1000 tags
  Memory: 1000 × 32 = 32KB
  Allocation rate: ~10/second (typical)
  Reuse rate: ~95%

Pool for Messages (moderate churn):
  Pool size: 100 messages
  Memory: 100 × 500 = 50KB
  Allocation rate: ~50/second
  Reuse rate: ~80%
```

### 2. String Interning

```
Device ID interning:
  Unique devices: ~10 per user
  String size: 16 bytes each
  Without interning: 16 bytes per reference
  With interning: 4 bytes per reference (pointer to pool)

  Savings in OR-Set with 100 items, 2 tags each:
    Without: 200 × 16 = 3,200 bytes
    With: (10 × 16) + (200 × 4) = 960 bytes
    Reduction: 70%
```

### 3. Compression Strategies

```
CRDT State compression:
  Method: LZ4 (fast compression)
  Ratio: 2-3x for typical states
  Latency: <1ms compress, <0.5ms decompress

  Watchlist (100 items):
    Uncompressed: 10KB
    Compressed: 3-4KB
    Network savings: 6-7KB

  Only compress when:
    - State > 1KB
    - Network quality is POOR or FAIR
    - Storage (always compress in IndexedDB)
```

### 4. Garbage Collection Tuning

```
CRDT tombstone cleanup:
  Frequency: Every 24 hours
  Retention: 7 days

  Before GC (1 week of removals):
    Tombstones: ~200 tags
    Memory: 200 × 32 = 6.4KB

  After GC:
    Tombstones: ~20 tags (last 24 hours)
    Memory: 20 × 32 = 640 bytes
    Reduction: 90%

Offline queue cleanup:
  Frequency: On successful sync
  Retention: None (remove synced)

  Before sync:
    Queue: 100 changes
    Memory: 100 × 200 = 20KB

  After sync:
    Queue: 0-5 changes (failed only)
    Memory: 1KB
    Reduction: 95%
```

## Storage Layout

### 1. IndexedDB Schema

```
DATABASE: media-gateway-sync
VERSION: 1

OBJECT STORES:
  1. local_changes
     Key: change_id (string)
     Indexes:
       - status (for pending query)
       - timestamp (for ordering)
     Size: ~200 bytes per entry

  2. crdt_states
     Key: resource_type:resource_id (string)
     Indexes:
       - last_modified (for cleanup)
       - resource_type (for type-based queries)
     Size: Variable (50-500 bytes typical)

  3. presence_cache
     Key: user_id (string)
     Value: PresenceState
     Size: ~2KB per user
     TTL: 5 minutes

  4. message_cache
     Key: message_id (string)
     Indexes:
       - channel
       - timestamp
     Size: ~500 bytes per message
     Retention: 1 hour (for deduplication)

Total storage estimate:
  Light user: <1MB
  Normal user: 1-5MB
  Heavy user: 5-20MB
  Maximum: 50MB (quota limit)
```

### 2. Memory vs Storage Tradeoff

```
Memory (RAM):
  Advantages:
    - Fast access (O(1) hash map)
    - No serialization overhead
    - Supports complex data structures

  Disadvantages:
    - Lost on page reload
    - Limited size (~100MB practical)
    - GC pressure

  Best for:
    - Active CRDT states
    - Pending commands
    - LRU cache
    - Current presence

Storage (IndexedDB):
  Advantages:
    - Persistent across sessions
    - Larger capacity (50MB+)
    - Minimal GC impact

  Disadvantages:
    - Slower access (~5-10ms)
    - Serialization required
    - Async API

  Best for:
    - Offline queue
    - Historical CRDT states
    - Message history
    - User preferences
```

## Memory Profiling

### Typical Session Memory Usage

```
Component breakdown:
┌─────────────────────────────────────────────────┐
│ PubNub SDK                     5KB              │
├─────────────────────────────────────────────────┤
│ Presence (5 devices)           1KB              │
├─────────────────────────────────────────────────┤
│ Active CRDT states (10)       20KB              │
├─────────────────────────────────────────────────┤
│ LRU cache (100 entries)       50KB              │
├─────────────────────────────────────────────────┤
│ Offline queue (50 changes)    10KB              │
├─────────────────────────────────────────────────┤
│ Remote control (5 pending)     2KB              │
├─────────────────────────────────────────────────┤
│ Message batcher                 1KB             │
├─────────────────────────────────────────────────┤
│ Performance monitor             5KB             │
├─────────────────────────────────────────────────┤
│ Miscellaneous overhead         6KB              │
└─────────────────────────────────────────────────┘
Total:                         ~100KB

Peak session (heavy usage):
  - Presence: 10 devices          2KB
  - CRDT states: 50 items       100KB
  - LRU cache: 1000 entries     500KB
  - Offline queue: 500 changes  100KB
  - Object pools                 82KB
Total:                         ~784KB
```

### Memory Leak Prevention

```
Common leak sources:
1. Event listeners not removed
   - Fix: WeakMap for handlers
   - Fix: Explicit cleanup on disconnect

2. Timer references not cleared
   - Fix: Track timer IDs
   - Fix: Clear on component unmount

3. Circular references in CRDTs
   - Fix: Weak references where possible
   - Fix: Explicit cleanup in GC

4. Growing message cache
   - Fix: LRU eviction
   - Fix: TTL-based cleanup

5. Unbounded offline queue
   - Fix: Cap at 1000 entries
   - Fix: FIFO eviction when full
```

## Performance Characteristics

### Memory Access Patterns

```
Hot paths (accessed frequently):
  - Current HLC: 100+ accesses/second
  - Active CRDT state: 10-50 accesses/second
  - LRU cache: 20-100 accesses/second

  Optimization: Keep in L1/L2 cache
  Size constraint: <64KB total

Cold paths (accessed rarely):
  - Historical tombstones: <1 access/hour
  - Old offline changes: <1 access/session

  Optimization: Can be in storage
  No memory constraint
```

This data structures documentation provides the foundation for efficient implementation with predictable memory usage and optimal performance characteristics.
