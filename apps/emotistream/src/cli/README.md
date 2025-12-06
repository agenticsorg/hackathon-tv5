# EmotiStream CLI Demo

Interactive demonstration of the emotion-aware content recommendation system with reinforcement learning.

## Quick Start

```bash
# Run the demo
npm run demo

# Or using tsx directly
tsx src/cli/index.ts
```

## Demo Flow

### Session Structure (3 iterations)

1. **🎭 Emotional State Detection**
   - Describe how you're feeling
   - System analyzes valence, arousal, and stress
   - Primary emotion identified with confidence

2. **🎯 Desired State Prediction**
   - System predicts optimal emotional target
   - Shows intensity and reasoning
   - Visualizes target vs current state

3. **🎬 AI-Powered Recommendations**
   - 5 personalized content recommendations
   - Q-values from reinforcement learning
   - Similarity scores from emotional profiling
   - Mix of exploration and exploitation

4. **📺 Content Selection & Viewing**
   - Choose from recommendations
   - Simulated viewing with progress bar
   - Completion tracking

5. **💬 Feedback & Learning**
   - Provide feedback (text/rating/emoji)
   - System calculates reward
   - Q-values updated using Q-learning
   - Policy improves over time

6. **📊 Learning Progress**
   - Total experiences
   - Average reward
   - Exploration rate
   - Convergence score

## Feedback Methods

### 1. Text Feedback (Most Accurate)
```
"I feel much more relaxed and calm now"
"That was uplifting and made me happy"
```

### 2. Star Rating
- ⭐⭐⭐⭐⭐ (5) - Excellent
- ⭐⭐⭐⭐ (4) - Good
- ⭐⭐⭐ (3) - Okay
- ⭐⭐ (2) - Poor
- ⭐ (1) - Very Poor

### 3. Emoji Feedback
- 😊 Happy
- 😌 Relaxed
- 😐 Neutral
- 😢 Sad
- 😡 Angry
- 😴 Sleepy

## Example Session

```
╔═══════════════════════════════════════════════════════════════════╗
║        EmotiStream Nexus - AI-Powered Emotional Wellness         ║
╚═══════════════════════════════════════════════════════════════════╝

Session 1 of 3

═══ Step 1: Emotional State Detection ═══

How are you feeling?
> "I'm feeling stressed and overwhelmed from work today"

📊 Emotional State Analysis:
   Valence:  ████░░░░░░░░░░░░░░░░ -0.60 (negative)
   Arousal:  ████████████░░░░░░░░  0.20 (moderate)
   Stress:   ████████████████░░░░  0.80 (very high)
   Primary:  😰 STRESS (85% confidence)

═══ Step 2: Predicting Desired State ═══

🎯 Predicted Desired State:
   Target Valence:  ██████████████░░░░░░  0.30
   Target Arousal:  ████░░░░░░░░░░░░░░░░ -0.40
   Target Stress:   ████░░░░░░░░░░░░░░░░  0.20
   Intensity: SIGNIFICANT
   Reasoning: Focus on stress reduction and calming

═══ Step 3: AI-Powered Recommendations ═══

┌──┬──────────────────────────┬──────────┬────────────┬────────────┐
│# │Title                     │Q-Value   │Similarity  │Type        │
├──┼──────────────────────────┼──────────┼────────────┼────────────┤
│1 │Ocean Waves & Sunset      │0.750     │0.892       │✓ Exploit   │
│2 │Peaceful Mountain Medit...│0.720     │0.876       │✓ Exploit   │
│3 │Classical Music for Str...│0.680     │0.845       │✓ Exploit   │
│4 │Beautiful Earth: Travel...│0.420     │0.623       │🔍 Explore  │
│5 │Guided Mindfulness Jour...│0.710     │0.889       │✓ Exploit   │
└──┴──────────────────────────┴──────────┴────────────┴────────────┘

Choose content: Ocean Waves & Sunset

═══ Step 4: Viewing Experience ═══

📺 Now watching: Ocean Waves & Sunset
████████████████████ 100%
✓ Viewing complete

═══ Step 5: Feedback & Learning ═══

Choose feedback method: 💬 Text feedback
Describe how you feel now: "I feel much more relaxed and calm now"

🎯 Reinforcement Learning Update:

   Content: Ocean Waves & Sunset
   Type: Exploitation

   📊 Emotional Journey:
   Before:  V:-0.60 A: 0.20 S:0.80 😰
   After:   V: 0.50 A:-0.40 S:0.20 😌
   Target:  V: 0.30 A:-0.40 S:0.20 🎯

   💰 Reward Calculation:
   ████████████████████ 0.782
   Excellent match! System learning strongly.

   📈 Q-Value Update:
   Old Q-value: 0.7500
   New Q-value: 0.7532
   Change:      +0.0032

   ✓ Policy successfully updated

═══ Step 6: Learning Progress ═══

📚 Learning Progress:

   Total Experiences: 1
   Average Reward:    ████████████████████ 0.782
   Exploration Rate:  ████░░░░░░░░░░░░░░░░ 20.0%
   Convergence:       ████░░░░░░░░░░░░░░░░ 15.6%

   💡 Interpretation:
   ✓ System is learning effectively
   Recommendations are consistently good
```

## Architecture

```
src/cli/
├── index.ts                    # Entry point
├── demo.ts                     # Main flow orchestration
├── prompts.ts                  # User input prompts
├── display/
│   ├── welcome.ts             # Welcome screen
│   ├── emotion.ts             # Emotion visualization
│   ├── recommendations.ts     # Recommendation table
│   ├── reward.ts              # Reward update display
│   └── learning.ts            # Learning progress
└── mock/
    ├── emotion-detector.ts    # Mock emotion detection
    ├── recommendation-engine.ts # Mock RL recommendations
    └── feedback-processor.ts   # Mock Q-learning updates
```

## Mock Content Catalog

1. **Peaceful Mountain Meditation** - Nature, calm
2. **Laughter Therapy: Stand-Up Special** - Comedy, uplifting
3. **The Art of Resilience** - Drama, inspirational
4. **Adrenaline Rush: Extreme Sports** - Action, exciting
5. **Ocean Waves & Sunset** - Relaxation, deep calm
6. **Classical Music for Stress Relief** - Music, therapy
7. **Stories of Hope and Triumph** - Documentary, inspirational
8. **Heartwarming Family Sitcom** - Comedy, gentle
9. **Guided Mindfulness Journey** - Wellness, meditation
10. **Beautiful Earth: Travel Documentary** - Travel, light adventure

## Key Features

✅ **Emotion Detection**
- Text analysis for valence, arousal, stress
- Primary emotion classification
- Confidence scoring

✅ **Q-Learning Recommendations**
- State-action Q-values
- ε-greedy exploration (20%)
- Combined Q-value + similarity scoring

✅ **Multi-Factor Rewards**
- Direction alignment (cosine similarity)
- Magnitude of emotional change
- Proximity to target state
- Completion bonus/penalty

✅ **Learning Metrics**
- Total experiences
- Average reward (EMA)
- Exploration rate decay
- Convergence tracking

✅ **Rich Visualization**
- ASCII progress bars
- Color-coded metrics
- Formatted tables
- Real-time spinners

## Technical Details

### Q-Learning Update
```
Q(s,a) ← Q(s,a) + α[r - Q(s,a)]
```
- Learning rate α = 0.1
- No discount (terminal state)

### Reward Calculation
```
reward = direction × 0.6 + magnitude × 0.4 + proximity_bonus
```
- Direction: Cosine similarity of emotional change
- Magnitude: Distance traveled in emotional space
- Proximity: Bonus for reaching target (max 0.2)

### Exploration Decay
```
ε(t+1) = max(0.05, ε(t) × 0.99)
```
- Initial: 20%
- Minimum: 5%

## Integration Points

To connect to the real system:

1. Replace `MockEmotionDetector` with Gemini-based detector
2. Replace `MockRecommendationEngine` with `RLPolicyEngine`
3. Replace `MockFeedbackProcessor` with real reward calculator
4. Load content from `MockCatalogGenerator`
5. Persist Q-values to AgentDB

## Troubleshooting

### Demo won't start
```bash
# Install dependencies
npm install

# Build TypeScript
npm run build

# Run demo
npm run demo
```

### TypeScript errors
```bash
# Clean build
rm -rf dist/
npm run build
```

### Import errors
Make sure all files use `.js` extensions in imports (ESM):
```typescript
import { DemoFlow } from './demo.js';
```

## Next Steps

1. Run the demo to see the full flow
2. Try different emotional states
3. Observe Q-value updates over time
4. See exploration vs exploitation balance
5. Check learning progress convergence

Enjoy the EmotiStream experience! 🎬✨
