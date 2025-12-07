# OR-Set (Observed-Remove Set) Pseudocode

## Overview
Provides conflict-free replicated set with add-wins semantics using unique tags. Ideal for watchlist management where additions should take precedence over removals.

## Data Structures

```
STRUCTURE ORSet<T>:
    elements: Map<T, Set<UniqueTag>>     // element -> set of unique tags
    tombstones: Map<T, Set<UniqueTag>>   // removed tags

STRUCTURE UniqueTag:
    device_id: string
    hlc: HLC
    sequence: uint32      // Per-device sequence number

STRUCTURE ORSetAdd<T>:
    element: T
    tag: UniqueTag

STRUCTURE ORSetRemove<T>:
    element: T
    observed_tags: Set<UniqueTag>
```

## Core Algorithms

### 1. OR-Set Creation

```
ALGORITHM: CreateORSet
INPUT: none
OUTPUT: set (ORSet<T>)

BEGIN
    set ← ORSet<T>()
    set.elements ← EmptyMap()
    set.tombstones ← EmptyMap()

    RETURN set
END
```

### 2. Add Element (Local)

```
ALGORITHM: AddToORSet
INPUT:
    set (ORSet<T>),
    element (T),
    device_id (string),
    current_hlc (HLC),
    sequence_counter (uint32)
OUTPUT: updated_set (ORSet<T>), add_message (ORSetAdd<T>), new_sequence (uint32)

BEGIN
    // Create unique tag for this add operation
    tag ← UniqueTag()
    tag.device_id ← device_id
    tag.hlc ← IncrementHLC(current_hlc, GetWallClock())
    tag.sequence ← sequence_counter + 1

    // Add element with unique tag
    IF NOT set.elements.has(element) THEN
        set.elements.set(element, EmptySet())
    END IF

    set.elements.get(element).add(tag)

    // Create message for broadcast
    add_message ← ORSetAdd<T>()
    add_message.element ← element
    add_message.tag ← tag

    RETURN set, add_message, tag.sequence
END
```

### 3. Remove Element (Local)

```
ALGORITHM: RemoveFromORSet
INPUT:
    set (ORSet<T>),
    element (T)
OUTPUT: updated_set (ORSet<T>), remove_message (ORSetRemove<T>)

BEGIN
    remove_message ← ORSetRemove<T>()
    remove_message.element ← element

    // Only remove if element exists
    IF NOT set.elements.has(element) THEN
        RETURN set, null  // Element not in set
    END IF

    // Observe all current tags for this element
    observed_tags ← set.elements.get(element).clone()
    remove_message.observed_tags ← observed_tags

    // Move tags to tombstones
    IF NOT set.tombstones.has(element) THEN
        set.tombstones.set(element, EmptySet())
    END IF

    FOR EACH tag IN observed_tags DO
        set.tombstones.get(element).add(tag)
    END FOR

    // Remove from elements map
    set.elements.delete(element)

    RETURN set, remove_message
END
```

### 4. Merge Add Operation

```
ALGORITHM: MergeORSetAdd
INPUT:
    set (ORSet<T>),
    add_message (ORSetAdd<T>)
OUTPUT: updated_set (ORSet<T>), was_added (boolean)

BEGIN
    element ← add_message.element
    tag ← add_message.tag

    // Check if tag already in tombstones (concurrent remove)
    IF set.tombstones.has(element) THEN
        IF set.tombstones.get(element).contains(tag) THEN
            // Tag was already removed - ignore add
            RETURN set, false
        END IF
    END IF

    // Add element with tag
    IF NOT set.elements.has(element) THEN
        set.elements.set(element, EmptySet())
    END IF

    // Check if tag already exists (idempotence)
    IF set.elements.get(element).contains(tag) THEN
        RETURN set, false  // Already added
    END IF

    // Add new tag
    set.elements.get(element).add(tag)

    RETURN set, true
END
```

### 5. Merge Remove Operation

```
ALGORITHM: MergeORSetRemove
INPUT:
    set (ORSet<T>),
    remove_message (ORSetRemove<T>)
OUTPUT: updated_set (ORSet<T>), was_removed (boolean)

BEGIN
    element ← remove_message.element
    observed_tags ← remove_message.observed_tags

    was_removed ← false

    // Add observed tags to tombstones
    IF NOT set.tombstones.has(element) THEN
        set.tombstones.set(element, EmptySet())
    END IF

    FOR EACH tag IN observed_tags DO
        set.tombstones.get(element).add(tag)

        // Remove tag from elements if present
        IF set.elements.has(element) THEN
            IF set.elements.get(element).contains(tag) THEN
                set.elements.get(element).remove(tag)
                was_removed ← true
            END IF
        END IF
    END FOR

    // Clean up empty element entry
    IF set.elements.has(element) AND set.elements.get(element).isEmpty() THEN
        set.elements.delete(element)
    END IF

    RETURN set, was_removed
END
```

### 6. Query Operations

```
ALGORITHM: ORSetContains
INPUT: set (ORSet<T>), element (T)
OUTPUT: boolean

BEGIN
    IF NOT set.elements.has(element) THEN
        RETURN false
    END IF

    RETURN NOT set.elements.get(element).isEmpty()
END

ALGORITHM: ORSetSize
INPUT: set (ORSet<T>)
OUTPUT: count (integer)

BEGIN
    count ← 0

    FOR EACH (element, tags) IN set.elements DO
        IF NOT tags.isEmpty() THEN
            count ← count + 1
        END IF
    END FOR

    RETURN count
END

ALGORITHM: ORSetToArray
INPUT: set (ORSet<T>)
OUTPUT: elements (array of T)

BEGIN
    elements ← []

    FOR EACH (element, tags) IN set.elements DO
        IF NOT tags.isEmpty() THEN
            elements.append(element)
        END IF
    END FOR

    RETURN elements
END
```

### 7. Watchlist Management

```
ALGORITHM: AddToWatchlist
INPUT:
    watchlist (ORSet<MediaItem>),
    media_item (MediaItem),
    device_id (string),
    hlc (HLC),
    sequence (uint32)
OUTPUT: updated_watchlist, add_message, new_sequence

BEGIN
    // Validate media item
    IF media_item.id is empty THEN
        RETURN error("Invalid media item")
    END IF

    // Add to OR-Set
    updated_watchlist, add_message, new_sequence ← AddToORSet(
        watchlist,
        media_item,
        device_id,
        hlc,
        sequence
    )

    LogInfo("Added to watchlist", media_item.id, device_id)

    RETURN updated_watchlist, add_message, new_sequence
END

ALGORITHM: RemoveFromWatchlist
INPUT:
    watchlist (ORSet<MediaItem>),
    media_item (MediaItem)
OUTPUT: updated_watchlist, remove_message

BEGIN
    // Check if item exists
    IF NOT ORSetContains(watchlist, media_item) THEN
        RETURN watchlist, null  // Not in watchlist
    END IF

    // Remove from OR-Set
    updated_watchlist, remove_message ← RemoveFromORSet(watchlist, media_item)

    LogInfo("Removed from watchlist", media_item.id)

    RETURN updated_watchlist, remove_message
END

ALGORITHM: GetWatchlistItems
INPUT: watchlist (ORSet<MediaItem>), limit (integer), offset (integer)
OUTPUT: items (array of MediaItem)

BEGIN
    all_items ← ORSetToArray(watchlist)

    // Sort by most recently added (newest HLC first)
    sorted_items ← SortByNewestTag(all_items, watchlist)

    // Apply pagination
    start_index ← offset
    end_index ← MIN(offset + limit, Length(sorted_items))

    paginated_items ← sorted_items.slice(start_index, end_index)

    RETURN paginated_items
END

SUBROUTINE: SortByNewestTag
INPUT: items (array of T), set (ORSet<T>)
OUTPUT: sorted_items (array of T)

BEGIN
    // For each item, find newest tag
    item_timestamps ← []

    FOR EACH item IN items DO
        tags ← set.elements.get(item)
        newest_tag ← FindNewestTag(tags)
        item_timestamps.append({item: item, hlc: newest_tag.hlc})
    END FOR

    // Sort by HLC descending
    sorted_items ← SortByHLCDescending(item_timestamps)

    RETURN sorted_items.map(entry -> entry.item)
END

SUBROUTINE: FindNewestTag
INPUT: tags (Set<UniqueTag>)
OUTPUT: newest_tag (UniqueTag)

BEGIN
    newest_tag ← null

    FOR EACH tag IN tags DO
        IF newest_tag is null OR CompareHLC(tag.hlc, newest_tag.hlc) > 0 THEN
            newest_tag ← tag
        END IF
    END FOR

    RETURN newest_tag
END
```

### 8. State Merge (Full Synchronization)

```
ALGORITHM: MergeORSets
INPUT: local_set (ORSet<T>), remote_set (ORSet<T>)
OUTPUT: merged_set (ORSet<T>)

BEGIN
    merged_set ← CreateORSet()

    // Merge elements
    all_elements ← Union(local_set.elements.keys(), remote_set.elements.keys())

    FOR EACH element IN all_elements DO
        local_tags ← local_set.elements.get(element) OR EmptySet()
        remote_tags ← remote_set.elements.get(element) OR EmptySet()

        // Union of all tags
        merged_tags ← Union(local_tags, remote_tags)

        // Get tombstones for this element
        local_tombstones ← local_set.tombstones.get(element) OR EmptySet()
        remote_tombstones ← remote_set.tombstones.get(element) OR EmptySet()

        // Union of all tombstones
        merged_tombstones ← Union(local_tombstones, remote_tombstones)

        // Remove tombstoned tags from active tags
        active_tags ← Difference(merged_tags, merged_tombstones)

        // Add to merged set if active tags remain
        IF NOT active_tags.isEmpty() THEN
            merged_set.elements.set(element, active_tags)
        END IF

        // Merge tombstones
        IF NOT merged_tombstones.isEmpty() THEN
            merged_set.tombstones.set(element, merged_tombstones)
        END IF
    END FOR

    RETURN merged_set
END
```

### 9. Garbage Collection

```
ALGORITHM: CompactORSet
INPUT: set (ORSet<T>), retention_period (milliseconds)
OUTPUT: compacted_set (ORSet<T>), removed_count (integer)

BEGIN
    current_time ← GetWallClock()
    removed_count ← 0

    // Remove old tombstones
    FOR EACH (element, tombstone_tags) IN set.tombstones DO
        tags_to_keep ← EmptySet()

        FOR EACH tag IN tombstone_tags DO
            age ← current_time - tag.hlc.physical_time

            IF age < retention_period THEN
                tags_to_keep.add(tag)
            ELSE
                removed_count ← removed_count + 1
            END IF
        END FOR

        IF tags_to_keep.isEmpty() THEN
            set.tombstones.delete(element)
        ELSE
            set.tombstones.set(element, tags_to_keep)
        END IF
    END FOR

    LogInfo("Compacted OR-Set", removed_count, "tombstones removed")

    RETURN set, removed_count
END
```

### 10. Serialization

```
ALGORITHM: SerializeORSet
INPUT: set (ORSet<T>)
OUTPUT: json_string (string)

BEGIN
    elements_obj ← {}
    tombstones_obj ← {}

    // Serialize elements
    FOR EACH (element, tags) IN set.elements DO
        elements_obj[SerializeElement(element)] ← SerializeTagSet(tags)
    END FOR

    // Serialize tombstones
    FOR EACH (element, tags) IN set.tombstones DO
        tombstones_obj[SerializeElement(element)] ← SerializeTagSet(tags)
    END FOR

    json_object ← {
        "elements": elements_obj,
        "tombstones": tombstones_obj
    }

    RETURN ToJSON(json_object)
END

SUBROUTINE: SerializeTagSet
INPUT: tags (Set<UniqueTag>)
OUTPUT: array of objects

BEGIN
    serialized ← []

    FOR EACH tag IN tags DO
        serialized.append({
            "device_id": tag.device_id,
            "hlc": SerializeHLC(tag.hlc),
            "sequence": tag.sequence
        })
    END FOR

    RETURN serialized
END
```

## Complexity Analysis

### Time Complexity
- `CreateORSet`: O(1)
- `AddToORSet`: O(1) average, O(n) worst case (n = tags per element)
- `RemoveFromORSet`: O(t) where t = number of tags for element
- `MergeORSetAdd`: O(1) average
- `MergeORSetRemove`: O(t)
- `ORSetContains`: O(1) average
- `ORSetSize`: O(e) where e = number of unique elements
- `ORSetToArray`: O(e)
- `MergeORSets`: O(e × t)
- `CompactORSet`: O(e × t)

### Space Complexity
- ORSet: O(e × t) where e = elements, t = average tags per element
- Typical watchlist: O(100 × 3) = O(300) tags for 100 items
- Add operation: O(1)
- Remove operation: O(t)
- Full state sync: O(e × t)

## Properties

### Add-Wins Semantics
- Concurrent add and remove: add wins
- Element appears in set if at least one tag not in tombstones

### Strong Eventual Consistency
- All replicas converge to same set
- Order of operations doesn't matter

### Idempotence
- Applying same add/remove multiple times has same effect

### Commutativity
- Operations can be applied in any order
- MergeORSets is commutative and associative

## Usage Example

```
// Device A: Add movie to watchlist
add_msg ← AddToWatchlist(watchlist_a, movie_123, "device-a", hlc_a, seq_a)
Publish("user.123.watchlist", add_msg)

// Device B: Concurrently removes same movie
remove_msg ← RemoveFromWatchlist(watchlist_b, movie_123)
Publish("user.123.watchlist", remove_msg)

// Device A receives remove (device B observed old tag)
MergeORSetRemove(watchlist_a, remove_msg)
// Result: movie_123 still in set (new tag not observed)

// Device B receives add (new tag after remove)
MergeORSetAdd(watchlist_b, add_msg)
// Result: movie_123 in set (add-wins)
```

## Performance Characteristics

- **Latency**: <5ms for add/remove operations
- **Network**: 80-150 bytes per operation
- **Memory**: ~24 bytes per tag (device_id + HLC + sequence)
- **Typical Watchlist**: 100 items × 2 tags = 4.8KB
- **Convergence**: Immediate upon receiving all operations

## Edge Cases

1. **Concurrent Add/Remove**: Add wins (by design)
2. **Duplicate Tags**: Prevented by HLC + device_id + sequence
3. **Tombstone Growth**: Mitigated by garbage collection
4. **Element Equality**: Requires proper equals/hash implementation
5. **Network Partition**: Full convergence on merge
