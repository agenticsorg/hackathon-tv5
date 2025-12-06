# EmotiStream User Guide

Welcome to EmotiStream - your emotion-aware content recommendation system!

## What is EmotiStream?

EmotiStream learns your emotional preferences and recommends content (movies, series, music, meditation) based on how you're feeling and how you want to feel. The more you use it, the smarter it gets at understanding what content works best for your emotional journey.

## Quick Start

### 1. Start the Application

**Option A: Interactive CLI Demo**
```bash
cd apps/emotistream
npm install
npm run start:cli
```

**Option B: REST API Server**
```bash
cd apps/emotistream
npm install
npm run start:api
```

The API server runs at `http://localhost:3000` by default.

### 2. Configure (Optional)

Copy the example environment file:
```bash
cp .env.example .env
```

Add your Gemini API key for real emotion detection:
```
GEMINI_API_KEY=your_api_key_here
```

Without a Gemini API key, EmotiStream uses mock emotion detection (still functional for demos).

---

## Using the CLI Demo

### Step 1: Describe How You Feel

When prompted, describe your current mood in natural language:
```
> How are you feeling? I'm stressed from work and feeling anxious about tomorrow
```

EmotiStream analyzes your input and maps it to an emotional state:
- **Valence**: How positive/negative you feel (-1 to +1)
- **Arousal**: How energetic/calm you feel (-1 to +1)
- **Stress**: Your stress level (0 to 1)

### Step 2: Tell Us Your Goal

Describe how you want to feel:
```
> How do you want to feel? Relaxed and peaceful, ready for sleep
```

### Step 3: Get Recommendations

EmotiStream recommends content personalized to your emotional journey:
```
╔══════════════════════════════════════════════════════════════╗
║  RECOMMENDATIONS FOR YOU                                      ║
╠══════════════════════════════════════════════════════════════╣
║  1. Deep Relaxation (meditation) - 15 min                    ║
║     Score: 0.87 | Expected mood: Calm, peaceful              ║
║                                                               ║
║  2. Tranquil Waves (music) - 45 min                          ║
║     Score: 0.82 | Expected mood: Serene, stress-free         ║
║                                                               ║
║  3. Ocean Deep (documentary) - 52 min                        ║
║     Score: 0.76 | Expected mood: Relaxed, interested         ║
╚══════════════════════════════════════════════════════════════╝
```

### Step 4: Watch & Provide Feedback

After watching content, tell EmotiStream how it affected you:
```
> How do you feel now? Much more relaxed, ready for bed
> Did you finish the content? Yes
> Rating (1-5): 5
```

This feedback trains the system to make better recommendations for you next time!

---

## Using the REST API

### Health Check
```bash
curl http://localhost:3000/health
```

### Analyze Your Emotions
```bash
curl -X POST http://localhost:3000/api/v1/emotion/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user123",
    "input": "I am feeling stressed and anxious",
    "desiredMood": "relaxed and calm"
  }'
```

### Get Recommendations
```bash
curl -X POST http://localhost:3000/api/v1/recommend \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user123",
    "currentState": {"valence": -0.3, "arousal": 0.6, "stress": 0.7},
    "desiredState": {"valence": 0.5, "arousal": -0.3},
    "limit": 3
  }'
```

### Submit Feedback
```bash
curl -X POST http://localhost:3000/api/v1/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user123",
    "contentId": "mock_meditation_001",
    "preState": {"valence": -0.3, "arousal": 0.6, "stress": 0.7},
    "postState": {"valence": 0.4, "arousal": -0.2, "stress": 0.2},
    "desiredState": {"valence": 0.5, "arousal": -0.3},
    "watchDuration": 900,
    "completed": true,
    "rating": 5
  }'
```

---

## Content Categories

EmotiStream includes diverse content types:

| Category | Examples | Best For |
|----------|----------|----------|
| **Movie** | Drama, Comedy, Thriller | Emotional experiences, entertainment |
| **Series** | Drama, Crime, Fantasy | Extended engagement, binge-watching |
| **Documentary** | Nature, History, Science | Learning, inspiration |
| **Music** | Classical, Jazz, Ambient | Background mood enhancement |
| **Meditation** | Guided, Breathing, Sleep | Stress relief, relaxation |
| **Short** | Animation, Comedy clips | Quick mood boost |

---

## How It Learns

EmotiStream uses **reinforcement learning** to personalize recommendations:

1. **State**: Your current emotional state (valence + arousal + stress)
2. **Action**: Content recommendation
3. **Reward**: How well the content moved you toward your desired state

The more feedback you provide, the better EmotiStream understands:
- Which content types work for your emotional transitions
- Your personal preferences and patterns
- Optimal content for different times and moods

---

## Tips for Best Results

1. **Be Descriptive**: The more detail in your mood description, the better the analysis
2. **Complete Content**: Finishing content provides clearer feedback signals
3. **Rate Honestly**: Your ratings directly improve future recommendations
4. **Use Regularly**: The system learns better with consistent usage
5. **Try Variety**: Exploring different content types helps discover new preferences

---

## Troubleshooting

### API not responding
```bash
# Check if server is running
curl http://localhost:3000/health

# Check port in use
lsof -i :3000
```

### Emotion detection not working
- Verify `GEMINI_API_KEY` is set in `.env`
- Check API key is valid and has quota
- Falls back to mock detection if API unavailable

### Recommendations seem random
- Normal for new users (exploration phase)
- Provide more feedback to train the model
- System needs ~10-20 interactions to personalize

---

## Support

- **Issues**: Report bugs at the project repository
- **Documentation**: See `docs/` folder for technical details
- **API Reference**: See `docs/API.md` for complete endpoint documentation
