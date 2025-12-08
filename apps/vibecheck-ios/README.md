# VibeCheck - Private Media Recommendations

**Your mood. Your data. Your recommendations.**

VibeCheck is a privacy-first iOS app that uses your Apple Health data to recommend movies and TV shows based on how you actually feel—not what algorithms think you should watch.

## The Problem

Every night, millions spend up to 45 minutes deciding what to watch. Netflix knows what you watched, but not how you feel. Your streaming platform optimizes for *their* retention, not *your* experience.

## The Solution

VibeCheck flips the model:

- **Your health data stays on YOUR device** - never uploaded, never shared
- **Real biometric signals** inform recommendations - HRV, sleep, activity
- **You own your preference profile** - export it, take it anywhere
- **Cross-platform recommendations** - not locked to one streaming service

## Features

### Mood Detection
- Heart Rate Variability → Stress level estimation
- Sleep data → Energy level assessment
- Step count → Activity classification
- Time of day → Context awareness

### Smart Recommendations
- Mood-based filtering: "comfort", "light", "engaging", "exciting", "calming"
- Genre preferences you control
- Runtime awareness (short episodes when you're tired)
- Platform availability (shows what's on YOUR subscriptions)

### Privacy by Design
- 100% on-device processing
- No accounts, no cloud, no tracking
- Export your data anytime (JSON)
- Revoke Health access in Settings

### Modern iOS UI
- iOS 18 Mesh Gradients for mood visualization
- Animated VibeRing mood indicator
- Interactive home screen widgets
- Scroll transitions and haptic feedback

## Requirements

- iOS 26 beta
- iPhone 12 Pro Max (or newer) with Apple Health
- Apple Watch recommended for HRV data
- Xcode 16+ with iOS 26 SDK (for building)

## Getting Started

1. Open `VibeCheck.xcodeproj` in Xcode
2. Select your development team in Signing & Capabilities
3. Build and run on a real device (HealthKit not available in Simulator)
4. Grant Health access when prompted
5. Check your vibe and get recommendations!

## Project Structure

```
VibeCheck/
├── App/
│   ├── VibeCheckApp.swift          # App entry point
│   └── ContentView.swift         # Tab navigation
├── Models/
│   ├── MoodState.swift           # Mood classification model
│   └── MediaItem.swift           # Media content model
├── Data/
│   ├── HealthKitManager.swift    # Health data access
│   └── LocalStore.swift          # SwiftData persistence
├── Engine/
│   ├── MoodClassifier.swift      # Biometric → Mood logic
│   └── RecommendationEngine.swift # Mood → Recommendations
├── Views/
│   ├── ForYouView.swift          # Main recommendations screen
│   ├── VibeCheckView.swift       # Health data dashboard
│   ├── WatchlistView.swift       # Saved items
│   ├── SettingsView.swift        # Preferences & privacy
│   └── Components/
│       ├── MoodMeshBackground.swift
│       ├── VibeRing.swift
│       ├── RecommendationCard.swift
│       ├── QuickMoodOverride.swift
│       └── VibeHeader.swift
├── Widget/
│   └── VibeWidget.swift          # Home screen widget
└── Resources/
    ├── Info.plist
    └── VibeCheck.entitlements
```

## How Mood Classification Works

### Energy Level
Based on sleep hours and activity:
- **Exhausted**: <5 hours sleep, sedentary day
- **Low**: 5-6 hours sleep
- **Moderate**: 6-7 hours sleep, some activity
- **High**: 7-8 hours sleep, active day
- **Wired**: High activity, moderate sleep

### Stress Level
Based on Heart Rate Variability (HRV):
- **Relaxed**: HRV > 70ms
- **Calm**: HRV 50-70ms
- **Neutral**: HRV 35-50ms
- **Tense**: HRV 20-35ms
- **Stressed**: HRV < 20ms

### Recommendation Hints
Mood combinations map to content types:
- **Comfort**: Feel-good shows, familiar rewatches (exhausted + any stress)
- **Gentle**: Low-intensity, no thrillers (low energy + calm)
- **Light**: Comedies, short episodes (any energy + high stress)
- **Engaging**: Moderate intensity (moderate energy + relaxed)
- **Exciting**: Action, adventure (high energy + relaxed)
- **Calming**: Documentaries, slow pacing (wired/any stress)

## Privacy

VibeCheck is built on a simple principle: **your health data is yours**.

- All processing happens on your iPhone
- We use Apple's HealthKit APIs with read-only access
- No data is ever sent to any server
- You can export all your preferences as JSON
- Deleting the app deletes all your data

Read more in Settings → Privacy.

## Built For

**Agentics Foundation TV5 Hackathon**
Track: Entertainment Discovery
Theme: Solve the 45-minute decision problem

## Technologies

- SwiftUI (iOS 18)
- SwiftData
- HealthKit
- WidgetKit
- SF Symbols 6

## License

Apache 2.0 - See [LICENSE](../../LICENSE)

---

**VibeCheck**: Because what you watch should match how you feel, not what an algorithm wants you to feel.
**https://gist.github.com/michaeloboyle/b768dd2a80b2dd521d4552d2d8f1e8a1**