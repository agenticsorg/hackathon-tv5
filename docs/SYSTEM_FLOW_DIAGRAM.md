# Content Discovery System Flow Diagram

This diagram illustrates the multi-agent content discovery and recommendation system with competitive advantages highlighted.

## Mermaid Diagram

```mermaid
graph TB
    %% Left Column
    N1([Generic / Session<br/>Strategic questioning])
    N2([Persona<br/>Profile updated])
    N3([Vectorized watch<br/>history])
    N4([Content aligned<br/>Ads])

    %% Center Column
    N5([Filtered content])
    N6([Welcome!<br/>Smart recommendations])
    N7([Content Watched])
    N8([Completed /<br/>Rated])
    N9([Recommend /<br/>Share])

    %% Right Column
    N10([Parent guidance])
    N11([Connect / follow /<br/>Groups])
    N12([Influence Stats])

    %% Edges - Main Flow
    N1 --> N2
    N2 --> N1
    N3 --> N2
    N2 --> N4
    N2 --> N6
    N10 --> N5
    N11 --> N5
    N5 --> N6
    N6 --> N7
    N7 --> N8
    N7 --> N9
    N8 --> N3
    N9 --> N11
    N11 --> N12

    %% Styling - Yellow for competitive advantages
    style N1 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N4 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N6 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N9 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N10 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N11 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000
    style N12 fill:#ffeb3b,stroke:#333,stroke-width:3px,color:#000

    %% Styling - White for normal components
    style N2 fill:#fff,stroke:#333,stroke-width:3px,color:#000
    style N3 fill:#fff,stroke:#333,stroke-width:3px,color:#000
    style N5 fill:#fff,stroke:#333,stroke-width:3px,color:#000
    style N7 fill:#fff,stroke:#333,stroke-width:3px,color:#000
    style N8 fill:#fff,stroke:#333,stroke-width:3px,color:#000
```

## Legend

🟨 **Yellow nodes** = Competitive advantage / Differentiator
⬜ **White nodes** = Normal system components

## System Flow Description

### Left Column (User Profile & Personalization)
1. **Generic/Session Strategic questioning** - Initial user understanding
2. **Persona (Profile updated)** - Centralized user profile
3. **Vectorized watch history** - ML-ready viewing data
4. **Content aligned Ads** - Personalized advertising

### Center Column (Content Pipeline)
1. **Filtered content** - Safety and preference-filtered content pool
2. **Welcome! Smart recommendations** - AI-powered recommendations
3. **Content Watched** - User viewing activity
4. **Completed/Rated** - Finished content with ratings
5. **Recommend/Share** - Social sharing features

### Right Column (Social & Safety)
1. **Parent guidance** - Content safety controls
2. **Connect/follow/Groups** - Social networking features
3. **Influence Stats** - Social influence metrics

## Competitive Advantages (Yellow Nodes)

1. **Strategic questioning** - Adaptive user profiling
2. **Smart recommendations** - AI-driven personalization
3. **Content aligned Ads** - Non-intrusive advertising
4. **Recommend/Share** - Viral growth mechanism
5. **Parent guidance** - Family-safe content filtering
6. **Connect/follow/Groups** - Social networking integration
7. **Influence Stats** - Gamification and engagement

## Key Feedback Loops

1. **Persona ↔ Strategic questioning** - Continuous profile refinement
2. **Content Watched → Rated → Watch history → Persona** - Learning loop
3. **Share → Groups → Filtered content** - Social discovery loop

---

**Created**: 2025-12-05
**Purpose**: System architecture visualization for multi-agent content discovery
