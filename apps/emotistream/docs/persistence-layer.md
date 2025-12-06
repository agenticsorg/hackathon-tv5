# EmotiStream Persistence Layer

## Overview

The persistence layer provides file-based storage for EmotiStream's reinforcement learning components. Data is stored as JSON files in the `./data/` directory with debounced writes to minimize I/O operations.

## Architecture

### Core Components

1. **FileStore** - Generic key-value store with file persistence
2. **QTableStore** - Stores Q-learning values for state-action pairs
3. **ExperienceStore** - Stores experience replay buffer
4. **UserProfileStore** - Stores user RL statistics and exploration rates

### Data Files

All data is persisted in the `./data/` directory:

```
data/
├── qtable.json           # Q-table entries (state-action values)
├── experiences.json      # Experience replay buffer
├── user-profiles.json    # User RL profiles
└── .gitkeep             # Ensures directory exists
```

## Features

### 1. Debounced Writes

- Writes are debounced by 1 second to avoid excessive I/O
- Multiple rapid updates are batched into a single write
- Call `flush()` for immediate write (useful for shutdown)

### 2. Graceful Error Handling

- Missing files are handled gracefully (starts fresh)
- Corrupt JSON is logged but doesn't crash the app
- Directory is created automatically if it doesn't exist

### 3. In-Memory Cache

- All data is kept in-memory for fast access
- File serves as persistent backup
- Reads are instant, writes are async

## Usage

### Q-Table Persistence

```typescript
import { QTable } from './src/rl/q-table';

const qTable = new QTable();

// Set Q-value (auto-persists)
await qTable.setQValue('state-sad', 'content-uplifting', 0.85);

// Get Q-value
const qValue = await qTable.getQValue('state-sad', 'content-uplifting');

// Update with Q-learning rule
await qTable.updateQValue(
  'state-sad',
  'content-uplifting',
  1.0,        // reward
  'state-happy', // next state
  0.1,        // learning rate
  0.9         // discount factor
);

// Epsilon-greedy action selection
const contentIds = ['content-1', 'content-2', 'content-3'];
const selected = await qTable.selectActions('state-sad', contentIds, 0.1);

// Force immediate save
qTable.flush();
```

### Experience Replay

```typescript
import { ExperienceStore } from './src/persistence/experience-store';
import { Experience } from './src/rl/types';

const store = new ExperienceStore();

// Add experience
const experience: Experience = {
  id: 'exp-123',
  userId: 'user-456',
  stateHash: 'state-sad',
  contentId: 'content-uplifting',
  reward: 0.85,
  nextStateHash: 'state-happy',
  timestamp: Date.now(),
  completed: true,
  watchDuration: 1800
};

await store.add(experience);

// Get user's experiences
const userExps = await store.getByUser('user-456');

// Get recent experiences
const recent = await store.getRecent('user-456', 10);

// Sample for replay
const batch = await store.sample('user-456', 32);

// Get high-reward experiences
const successful = await store.getHighReward('user-456', 0.7);

// Cleanup old data
const deleted = await store.cleanup(90 * 24 * 60 * 60 * 1000); // 90 days
```

### User Profiles

```typescript
import { UserProfileStore } from './src/persistence/user-profile-store';

const store = new UserProfileStore();

// Create or get profile
const profile = await store.getOrCreate('user-123');
console.log(profile.explorationRate); // 0.3 (initial)

// Increment experience count (auto-decays exploration)
await store.incrementExperiences('user-123');

// Update exploration rate manually
await store.updateExplorationRate('user-123', 0.15);

// Reset exploration
await store.resetExploration('user-123');

// Get global statistics
const stats = await store.getGlobalStats();
console.log(stats.totalUsers);
console.log(stats.avgExplorationRate);
console.log(stats.mostExperiencedUser);
```

## Q-Learning Algorithm

The Q-table implements the standard Q-learning update rule:

```
Q(s,a) ← Q(s,a) + α[r + γ max Q(s',a') - Q(s,a)]
```

Where:
- `s` = current state (emotional state hash)
- `a` = action (content ID)
- `r` = reward (user feedback)
- `s'` = next state (post-interaction emotional state)
- `α` = learning rate (default: 0.1)
- `γ` = discount factor (default: 0.9)

### Epsilon-Greedy Exploration

The exploration rate starts at 30% and decays using:

```
ε = max(ε_min, ε * 0.995^experiences)
```

Where:
- `ε` = exploration rate
- `ε_min` = 0.05 (minimum exploration)
- `experiences` = total user experiences

## Data Persistence Guarantees

### Persistence Across Restarts

Q-values, experiences, and user profiles survive server restarts:

```typescript
// Before restart
await qTable.setQValue('state-1', 'content-1', 0.75);
qTable.flush();

// After restart (new QTable instance)
const qTable2 = new QTable();
const value = await qTable2.getQValue('state-1', 'content-1');
console.log(value); // 0.75
```

### Graceful Shutdown

```typescript
import { flushAll } from './src/persistence';

// On server shutdown
process.on('SIGTERM', () => {
  flushAll(); // Save all pending changes
  process.exit(0);
});
```

## Testing

Run persistence tests:

```bash
npm test tests/persistence/
```

### Test Coverage

- Q-table persistence across instances
- Q-learning update rule correctness
- Epsilon-greedy action selection
- Experience replay sampling
- User profile exploration decay
- Statistics calculations
- File creation and JSON format
- Cleanup and deletion

## Performance Characteristics

### Time Complexity

- Get: O(1) - in-memory Map lookup
- Set: O(1) - in-memory Map set + debounced write
- Query: O(n) - linear scan with predicate
- Sample: O(n) - Fisher-Yates shuffle

### Space Complexity

- Memory: O(n) where n = total entries
- Disk: JSON file size (typically KB to low MB)

### Write Performance

- Debounced writes (1 second delay)
- Batches multiple updates into single write
- Typical write time: <10ms for 1000 entries

## Migration to AgentDB (Future)

When ready to migrate to AgentDB for better performance:

1. Install AgentDB: `npm install agentdb`
2. Update stores to use AgentDB client
3. Migrate data from JSON to AgentDB
4. Enable vector search and HNSW indexing

See `docs/specs/emotistream/IMPLEMENTATION-PLAN-ALPHA.md` Phase 2.1 for details.

## Troubleshooting

### Data not persisting

Check that:
1. `./data/` directory exists and is writable
2. `flush()` is called before shutdown
3. Debounce delay has elapsed (wait 1 second or call `flush()`)

### Corrupt JSON file

The store handles corrupt files gracefully by:
1. Logging a warning
2. Starting with fresh data
3. Overwriting corrupt file on next write

### Memory usage

If storing large amounts of data:
1. Implement periodic cleanup with `cleanup(maxAge)`
2. Limit experience buffer size per user
3. Consider migrating to AgentDB for compression

## API Reference

### QTable

| Method | Description |
|--------|-------------|
| `get(stateHash, contentId)` | Get Q-table entry |
| `getQValue(stateHash, contentId)` | Get Q-value (0 if not found) |
| `setQValue(stateHash, contentId, qValue)` | Set Q-value directly |
| `updateQValue(...)` | Update using Q-learning rule |
| `getStateActions(stateHash)` | Get all actions for state |
| `getBestAction(stateHash)` | Get highest Q-value action |
| `selectActions(...)` | Epsilon-greedy selection |
| `getStats()` | Get Q-table statistics |
| `flush()` | Force immediate save |
| `clear()` | Delete all Q-values |

### ExperienceStore

| Method | Description |
|--------|-------------|
| `add(experience)` | Add new experience |
| `get(experienceId)` | Get by ID |
| `getByUser(userId)` | Get all user experiences |
| `getRecent(userId, limit)` | Get recent N experiences |
| `getCompleted(userId)` | Get completed only |
| `getHighReward(userId, minReward)` | Get high-reward experiences |
| `sample(userId, size)` | Random sample for replay |
| `cleanup(maxAge)` | Delete old experiences |
| `getStats(userId)` | Get experience statistics |
| `flush()` | Force immediate save |
| `clear()` | Delete all experiences |

### UserProfileStore

| Method | Description |
|--------|-------------|
| `get(userId)` | Get user profile |
| `create(userId)` | Create with defaults |
| `getOrCreate(userId)` | Get or create |
| `updateExplorationRate(userId, rate)` | Set exploration rate |
| `incrementExperiences(userId)` | Increment count + decay ε |
| `incrementPolicyVersion(userId)` | Increment version |
| `resetExploration(userId)` | Reset to 0.3 |
| `getGlobalStats()` | Stats across all users |
| `flush()` | Force immediate save |
| `clear()` | Delete all profiles |

## Related Documentation

- [EmotiStream Implementation Plan](./specs/emotistream/IMPLEMENTATION-PLAN-ALPHA.md)
- [API Specification](./specs/emotistream/API-EmotiStream-MVP.md)
- [Q-Learning Architecture](./specs/emotistream/architecture/ARCH-RLPolicyEngine.md)
