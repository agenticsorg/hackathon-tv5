# CLI Demo - Pseudocode Specification

**Component**: Interactive CLI Demonstration
**Phase**: SPARC - Pseudocode
**Target Duration**: 3 minutes
**Purpose**: Hackathon presentation and live demonstration

---

## Table of Contents

1. [Main Demo Flow](#main-demo-flow)
2. [Display Functions](#display-functions)
3. [User Interaction Functions](#user-interaction-functions)
4. [Visualization Helpers](#visualization-helpers)
5. [Error Handling](#error-handling)
6. [Timing & Performance](#timing--performance)
7. [Rehearsal Checklist](#rehearsal-checklist)

---

## Main Demo Flow

### Primary Algorithm

```
ALGORITHM: runDemo
INPUT: none
OUTPUT: Promise<void>

CONSTANTS:
    DEMO_MODE = true
    MAX_ITERATIONS = 3
    DEFAULT_USER_ID = "demo-user-001"

BEGIN
    TRY
        // Phase 0: Initialization
        system ← InitializeSystem()
        userId ← DEFAULT_USER_ID
        iterationCount ← 0

        // Clear terminal and prepare display
        ClearTerminal()
        SetupColorScheme()

        // Phase 1: Welcome
        DisplayWelcome()
        WaitForKeypress("Press ENTER to start demonstration...")

        // Main demo loop
        REPEAT
            iterationCount ← iterationCount + 1

            // Phase 2: Emotional Input
            DisplaySectionHeader("Step 1: Emotional State Detection")
            emotionalText ← PromptEmotionalInput(iterationCount)

            // Phase 3: Emotion Detection
            DisplayLoadingSpinner("Analyzing emotional state...")
            emotionalState ← system.emotionDetector.analyze(emotionalText)
            Sleep(800) // Dramatic pause
            DisplayEmotionAnalysis(emotionalState)
            WaitForKeypress()

            // Phase 4: Desired State Prediction
            DisplaySectionHeader("Step 2: Predicting Desired State")
            DisplayLoadingSpinner("Calculating optimal emotional trajectory...")
            desiredState ← system.statePredictor.predict(emotionalState, userId)
            Sleep(600)
            DisplayDesiredState(desiredState)
            WaitForKeypress()

            // Phase 5: Generate Recommendations
            DisplaySectionHeader("Step 3: AI-Powered Recommendations")
            DisplayLoadingSpinner("Generating personalized recommendations...")
            recommendations ← system.recommendationEngine.getRecommendations(
                emotionalState,
                desiredState,
                userId,
                limit: 5
            )
            Sleep(700)
            DisplayRecommendations(recommendations, iterationCount)

            // Phase 6: Content Selection
            selectedContentId ← PromptContentSelection(recommendations)
            selectedContent ← FindContentById(recommendations, selectedContentId)

            // Phase 7: Simulate Viewing
            DisplaySectionHeader("Step 4: Viewing Experience")
            SimulateViewing(selectedContent)

            // Phase 8: Post-Viewing Feedback
            DisplaySectionHeader("Step 5: Feedback & Learning")
            feedbackInput ← PromptPostViewingFeedback()

            // Phase 9: Process Feedback & Display Reward
            DisplayLoadingSpinner("Processing feedback and updating model...")
            feedbackResponse ← system.feedbackProcessor.process(
                userId,
                selectedContent.id,
                emotionalState,
                feedbackInput
            )
            Sleep(500)
            DisplayRewardUpdate(feedbackResponse, selectedContent)

            // Phase 10: Show Learning Progress
            DisplaySectionHeader("Step 6: Learning Progress")
            DisplayLearningProgress(userId, iterationCount)
            WaitForKeypress()

            // Ask to continue
            IF iterationCount < MAX_ITERATIONS THEN
                shouldContinue ← PromptContinue()
                IF NOT shouldContinue THEN
                    BREAK
                END IF
                DisplayTransition()
            ELSE
                BREAK
            END IF

        UNTIL iterationCount >= MAX_ITERATIONS

        // Final summary
        DisplayFinalSummary(userId, iterationCount)
        DisplayThankYou()

    CATCH error
        HandleDemoError(error)
        DisplayErrorRecovery()
    END TRY
END
```

---

## Display Functions

### 1. Welcome Display

```
ALGORITHM: DisplayWelcome
INPUT: none
OUTPUT: void

CONSTANTS:
    LOGO_COLOR = "cyan"
    SUBTITLE_COLOR = "white"

BEGIN
    ClearTerminal()

    // ASCII Art Logo
    logo ← [
        "╔═══════════════════════════════════════════════════════╗",
        "║                                                       ║",
        "║     🎬  EmotiStream Nexus  🧠                        ║",
        "║                                                       ║",
        "║     Emotion-Driven Content Recommendations           ║",
        "║     Powered by Reinforcement Learning                ║",
        "║                                                       ║",
        "╚═══════════════════════════════════════════════════════╝"
    ]

    FOR EACH line IN logo DO
        Print(ColorText(line, LOGO_COLOR))
    END FOR

    PrintNewline(2)

    // Introduction text
    intro ← [
        "Welcome to the EmotiStream Nexus demonstration!",
        "",
        "This system:",
        "  • Detects your emotional state from text",
        "  • Predicts your desired emotional trajectory",
        "  • Recommends content using Q-Learning",
        "  • Learns from your feedback in real-time",
        "",
        "Duration: ~3 minutes",
        ""
    ]

    FOR EACH line IN intro DO
        Print(ColorText(line, SUBTITLE_COLOR))
    END FOR

    PrintNewline(1)
END
```

### 2. Emotion Analysis Display

```
ALGORITHM: DisplayEmotionAnalysis
INPUT: emotionalState (EmotionalState object)
OUTPUT: void

CONSTANTS:
    BAR_WIDTH = 20
    VALENCE_POSITIVE_COLOR = "green"
    VALENCE_NEGATIVE_COLOR = "red"
    AROUSAL_HIGH_COLOR = "yellow"
    AROUSAL_LOW_COLOR = "blue"
    STRESS_GRADIENT = ["green", "yellow", "orange", "red"]

BEGIN
    PrintSectionBorder("top")
    Print(ColorText("📊 Emotional State Detected:", "bold"))
    PrintNewline(1)

    // Valence display
    valenceBar ← CreateProgressBar(
        emotionalState.valence,
        min: -1,
        max: 1,
        width: BAR_WIDTH
    )
    valenceColor ← IF emotionalState.valence >= 0
                    THEN VALENCE_POSITIVE_COLOR
                    ELSE VALENCE_NEGATIVE_COLOR
    valenceLabel ← IF emotionalState.valence >= 0
                    THEN "positive"
                    ELSE "negative"

    Print("   Valence:  " + ColorText(valenceBar, valenceColor) +
          " " + FormatNumber(emotionalState.valence, 1) +
          " (" + valenceLabel + ")")

    // Arousal display
    arousalBar ← CreateProgressBar(
        emotionalState.arousal,
        min: -1,
        max: 1,
        width: BAR_WIDTH
    )
    arousalColor ← IF emotionalState.arousal >= 0
                   THEN AROUSAL_HIGH_COLOR
                   ELSE AROUSAL_LOW_COLOR
    arousalLevel ← GetArousalLevel(emotionalState.arousal)

    Print("   Arousal:  " + ColorText(arousalBar, arousalColor) +
          " " + FormatNumber(emotionalState.arousal, 1) +
          " (" + arousalLevel + ")")

    // Stress display
    stressBar ← CreateProgressBar(
        emotionalState.stress,
        min: 0,
        max: 1,
        width: BAR_WIDTH
    )
    stressColor ← GetStressColor(emotionalState.stress, STRESS_GRADIENT)
    stressLevel ← GetStressLevel(emotionalState.stress)

    Print("   Stress:   " + ColorText(stressBar, stressColor) +
          " " + FormatNumber(emotionalState.stress, 1) +
          " (" + stressLevel + ")")

    PrintNewline(1)

    // Primary emotion with emoji
    emoji ← GetEmotionEmoji(emotionalState.primaryEmotion)
    Print("   Primary:  " + emoji + " " +
          ColorText(emotionalState.primaryEmotion, "bold") +
          " (" + FormatPercentage(emotionalState.confidence) + " confidence)")

    // Secondary emotions if present
    IF emotionalState.secondaryEmotions.length > 0 THEN
        Print("   Secondary: " +
              FormatEmotionList(emotionalState.secondaryEmotions))
    END IF

    PrintNewline(1)
    PrintSectionBorder("bottom")
    PrintNewline(1)
END

SUBROUTINE: GetArousalLevel
INPUT: arousal (float -1 to 1)
OUTPUT: string

BEGIN
    IF arousal > 0.6 THEN RETURN "very excited"
    IF arousal > 0.2 THEN RETURN "moderate"
    IF arousal > -0.2 THEN RETURN "neutral"
    IF arousal > -0.6 THEN RETURN "calm"
    RETURN "very calm"
END

SUBROUTINE: GetStressLevel
INPUT: stress (float 0 to 1)
OUTPUT: string

BEGIN
    IF stress > 0.8 THEN RETURN "very high"
    IF stress > 0.6 THEN RETURN "high"
    IF stress > 0.4 THEN RETURN "moderate"
    IF stress > 0.2 THEN RETURN "low"
    RETURN "minimal"
END

SUBROUTINE: GetStressColor
INPUT: stress (float), gradient (array of colors)
OUTPUT: color string

BEGIN
    index ← Floor(stress * (gradient.length - 1))
    RETURN gradient[index]
END

SUBROUTINE: GetEmotionEmoji
INPUT: emotion (string)
OUTPUT: emoji string

BEGIN
    emojiMap ← {
        "sadness": "😔",
        "joy": "😊",
        "anger": "😠",
        "fear": "😨",
        "surprise": "😲",
        "disgust": "🤢",
        "neutral": "😐",
        "stress": "😰",
        "anxiety": "😟",
        "relaxation": "😌"
    }

    RETURN emojiMap[emotion] OR "🎭"
END
```

### 3. Desired State Display

```
ALGORITHM: DisplayDesiredState
INPUT: desiredState (DesiredState object)
OUTPUT: void

BEGIN
    PrintSectionBorder("top")
    Print(ColorText("🎯 Predicted Desired State:", "bold"))
    PrintNewline(1)

    // Target description
    Print("   Target:   " +
          ColorText(desiredState.targetDescription, "cyan"))

    // Target values
    valenceArrow ← IF desiredState.targetValence > 0
                   THEN "→ +"
                   ELSE "→ "
    arousalArrow ← IF desiredState.targetArousal > 0
                   THEN "→ +"
                   ELSE "→ "

    Print("   Valence:  " + valenceArrow +
          FormatNumber(desiredState.targetValence, 1))
    Print("   Arousal:  " + arousalArrow +
          FormatNumber(desiredState.targetArousal, 1))

    PrintNewline(1)

    // Reasoning
    Print("   Reasoning:")
    Print("   " + WrapText(desiredState.reasoning, 60, "   "))

    PrintNewline(1)

    // Confidence indicator
    confidenceBar ← CreateProgressBar(
        desiredState.confidence,
        min: 0,
        max: 1,
        width: 15
    )
    Print("   Confidence: " +
          ColorText(confidenceBar, "magenta") +
          " " + FormatPercentage(desiredState.confidence))

    PrintNewline(1)
    PrintSectionBorder("bottom")
    PrintNewline(1)
END
```

### 4. Recommendations Display

```
ALGORITHM: DisplayRecommendations
INPUT: recommendations (array of EmotionalRecommendation), iteration (integer)
OUTPUT: void

CONSTANTS:
    TABLE_WIDTH = 90
    COL_WIDTHS = [4, 30, 10, 10, 12, 24]

BEGIN
    PrintSectionBorder("top")
    Print(ColorText("📺 Top Recommendations:", "bold"))

    IF iteration > 1 THEN
        Print(ColorText("   (Notice: Q-values are updating based on your feedback!)", "yellow"))
    END IF

    PrintNewline(1)

    // Table header
    header ← CreateTableRow([
        "#",
        "Title",
        "Q-Value",
        "Similarity",
        "Effect",
        "Tags"
    ], COL_WIDTHS, "bold")

    Print(header)
    Print(CreateTableSeparator(COL_WIDTHS))

    // Table rows
    FOR i ← 0 TO recommendations.length - 1 DO
        rec ← recommendations[i]

        // Rank
        rank ← ToString(i + 1)

        // Title (truncate if needed)
        title ← TruncateText(rec.title, COL_WIDTHS[1] - 2)

        // Q-value with color
        qValue ← FormatNumber(rec.qValue, 3)
        qColor ← GetQValueColor(rec.qValue)
        qValueText ← ColorText(qValue, qColor)

        // Add change indicator if this is iteration 2+
        IF iteration > 1 AND rec.qValueChange != 0 THEN
            arrow ← IF rec.qValueChange > 0 THEN "⬆️" ELSE "⬇️"
            qValueText ← qValueText + " " + arrow
        END IF

        // Similarity
        similarity ← FormatNumber(rec.similarity, 2)

        // Emotional effect
        effect ← FormatEmotionalEffect(rec.emotionalEffect)

        // Tags (show first 2)
        tags ← FormatTagList(rec.tags, 2)

        // Add exploration indicator
        IF rec.isExploration THEN
            title ← title + " 🔍"
        END IF

        row ← CreateTableRow([
            rank,
            title,
            qValueText,
            similarity,
            effect,
            tags
        ], COL_WIDTHS, "normal")

        Print(row)
    END FOR

    Print(CreateTableSeparator(COL_WIDTHS))

    // Legend
    PrintNewline(1)
    Print("   Legend:")
    Print("   🔍 Exploration (new content to learn from)")
    Print("   ⬆️ Q-value increased  ⬇️ Q-value decreased")

    PrintNewline(1)
    PrintSectionBorder("bottom")
    PrintNewline(1)
END

SUBROUTINE: GetQValueColor
INPUT: qValue (float)
OUTPUT: color string

BEGIN
    IF qValue > 0.5 THEN RETURN "green"
    IF qValue > 0.2 THEN RETURN "yellow"
    IF qValue > 0 THEN RETURN "white"
    RETURN "gray"
END

SUBROUTINE: FormatEmotionalEffect
INPUT: effect (EmotionalEffect object)
OUTPUT: string

BEGIN
    parts ← []

    IF effect.valenceChange > 0.1 THEN
        parts.append("+V")
    ELSE IF effect.valenceChange < -0.1 THEN
        parts.append("-V")
    ELSE
        parts.append("~V")
    END IF

    IF effect.arousalChange > 0.1 THEN
        parts.append("+A")
    ELSE IF effect.arousalChange < -0.1 THEN
        parts.append("-A")
    ELSE
        parts.append("~A")
    END IF

    RETURN Join(parts, " ")
END

SUBROUTINE: FormatTagList
INPUT: tags (array of strings), maxCount (integer)
OUTPUT: string

BEGIN
    IF tags.length == 0 THEN RETURN "-"

    displayTags ← tags.slice(0, maxCount)
    result ← Join(displayTags, ", ")

    IF tags.length > maxCount THEN
        result ← result + " +"+ ToString(tags.length - maxCount)
    END IF

    RETURN result
END
```

### 5. Reward Update Display

```
ALGORITHM: DisplayRewardUpdate
INPUT: feedbackResponse (FeedbackResponse), content (Content)
OUTPUT: void

BEGIN
    PrintSectionBorder("top")
    Print(ColorText("🎉 Learning Update:", "bold"))
    PrintNewline(1)

    // Reward visualization
    reward ← feedbackResponse.reward
    rewardBar ← CreateProgressBar(reward, min: -1, max: 1, width: 30)
    rewardColor ← IF reward > 0.5 THEN "green"
                  ELSE IF reward > 0 THEN "yellow"
                  ELSE "red"

    Print("   Reward:    " +
          ColorText(rewardBar, rewardColor) +
          " " + ColorText(FormatNumber(reward, 2), rewardColor))

    PrintNewline(1)

    // Q-value change
    oldQValue ← feedbackResponse.oldQValue
    newQValue ← feedbackResponse.newQValue
    qValueDelta ← newQValue - oldQValue

    arrow ← IF qValueDelta > 0 THEN "→" ELSE "→"
    deltaColor ← IF qValueDelta > 0 THEN "green" ELSE "red"

    Print("   Q-value:   " +
          FormatNumber(oldQValue, 3) + " " + arrow + " " +
          ColorText(FormatNumber(newQValue, 3), deltaColor))

    IF qValueDelta != 0 THEN
        deltaText ← IF qValueDelta > 0
                    THEN "+" + FormatNumber(qValueDelta, 3)
                    ELSE FormatNumber(qValueDelta, 3)
        Print("              (change: " +
              ColorText(deltaText, deltaColor) + ")")
    END IF

    PrintNewline(1)

    // Emotional improvement
    IF feedbackResponse.emotionalImprovement != null THEN
        improvement ← feedbackResponse.emotionalImprovement
        improvementBar ← CreateProgressBar(
            improvement,
            min: -1,
            max: 1,
            width: 20
        )
        improvementColor ← IF improvement > 0 THEN "green" ELSE "red"

        Print("   Emotional Improvement: " +
              ColorText(improvementBar, improvementColor) +
              " " + FormatNumber(improvement, 2))

        PrintNewline(1)
    END IF

    // Learning message
    learningMessage ← GetLearningMessage(reward, qValueDelta)
    Print("   " + ColorText(learningMessage, "cyan"))

    PrintNewline(1)
    PrintSectionBorder("bottom")
    PrintNewline(1)
END

SUBROUTINE: GetLearningMessage
INPUT: reward (float), qValueDelta (float)
OUTPUT: string

BEGIN
    IF reward > 0.7 AND qValueDelta > 0 THEN
        RETURN "✅ Great feedback! This content will be prioritized."
    ELSE IF reward > 0.3 THEN
        RETURN "👍 Policy updated. Learning from your preferences."
    ELSE IF reward > 0 THEN
        RETURN "📊 Noted. Slight improvement in recommendation strategy."
    ELSE IF reward > -0.3 THEN
        RETURN "⚠️ Understood. Adjusting recommendations."
    ELSE
        RETURN "❌ Got it. This content will be deprioritized."
    END IF
END
```

### 6. Learning Progress Display

```
ALGORITHM: DisplayLearningProgress
INPUT: userId (string), iteration (integer)
OUTPUT: void

BEGIN
    // Fetch learning statistics
    stats ← GetLearningStatistics(userId)

    PrintSectionBorder("top")
    Print(ColorText("📈 Learning Progress:", "bold"))
    PrintNewline(1)

    // Basic statistics
    Print("   Total Experiences:  " +
          ColorText(ToString(stats.totalExperiences), "cyan"))
    Print("   Mean Reward:        " +
          FormatRewardWithTrend(stats.meanReward, stats.rewardTrend))
    Print("   Exploration Rate:   " +
          FormatPercentage(stats.explorationRate) +
          " (ε = " + FormatNumber(stats.epsilon, 2) + ")")

    PrintNewline(1)

    // Recent rewards visualization (ASCII chart)
    IF stats.recentRewards.length >= 5 THEN
        Print("   Recent Rewards (last 10):")
        PrintNewline(1)

        chart ← CreateASCIIChart(
            stats.recentRewards,
            width: 50,
            height: 8,
            min: -1,
            max: 1
        )

        Print(chart)
        PrintNewline(1)
    END IF

    // Learning insights
    IF iteration > 1 THEN
        insights ← GenerateLearningInsights(stats)
        IF insights.length > 0 THEN
            Print("   💡 Insights:")
            FOR EACH insight IN insights DO
                Print("      • " + insight)
            END FOR
            PrintNewline(1)
        END IF
    END IF

    PrintSectionBorder("bottom")
    PrintNewline(1)
END

SUBROUTINE: FormatRewardWithTrend
INPUT: meanReward (float), trend (float)
OUTPUT: string

BEGIN
    rewardText ← FormatNumber(meanReward, 2)

    trendArrow ← IF trend > 0.05 THEN "📈"
                 ELSE IF trend < -0.05 THEN "📉"
                 ELSE "➡️"

    color ← IF meanReward > 0.5 THEN "green"
            ELSE IF meanReward > 0 THEN "yellow"
            ELSE "red"

    RETURN ColorText(rewardText, color) + " " + trendArrow
END

SUBROUTINE: CreateASCIIChart
INPUT: values (array of float), width (int), height (int), min (float), max (float)
OUTPUT: string (multi-line chart)

CONSTANTS:
    BLOCKS = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"]

BEGIN
    chart ← []

    // Normalize values to chart height
    normalizedValues ← []
    FOR EACH value IN values DO
        normalized ← (value - min) / (max - min)
        barHeight ← Round(normalized * (BLOCKS.length - 1))
        normalizedValues.append(barHeight)
    END FOR

    // Create vertical bars
    barString ← "      "
    FOR EACH height IN normalizedValues DO
        block ← BLOCKS[height]
        color ← IF height > 6 THEN "green"
                ELSE IF height > 3 THEN "yellow"
                ELSE "red"
        barString ← barString + ColorText(block + block, color)
    END FOR

    chart.append(barString)

    // Add axis labels
    chart.append("      " + "─" * (values.length * 2))
    chart.append("      min=" + FormatNumber(min, 1) +
                 "  max=" + FormatNumber(max, 1))

    RETURN Join(chart, "\n")
END

SUBROUTINE: GenerateLearningInsights
INPUT: stats (LearningStatistics object)
OUTPUT: array of strings

BEGIN
    insights ← []

    // Exploration insight
    IF stats.explorationRate > 0.3 THEN
        insights.append("High exploration: discovering new content patterns")
    ELSE IF stats.explorationRate < 0.1 THEN
        insights.append("Focused exploitation: confident in recommendations")
    END IF

    // Reward trend insight
    IF stats.rewardTrend > 0.1 THEN
        insights.append("Positive trend: recommendations improving over time")
    ELSE IF stats.rewardTrend < -0.1 THEN
        insights.append("Needs calibration: gathering more preference data")
    END IF

    // Experience count insight
    IF stats.totalExperiences < 10 THEN
        insights.append("Early learning phase: building preference model")
    ELSE IF stats.totalExperiences > 50 THEN
        insights.append("Mature model: well-calibrated to your preferences")
    END IF

    RETURN insights
END
```

---

## User Interaction Functions

### 1. Emotional Input Prompt

```
ALGORITHM: PromptEmotionalInput
INPUT: iteration (integer)
OUTPUT: Promise<string>

CONSTANTS:
    DEFAULT_INPUTS = [
        "I'm feeling stressed after a long day",
        "I'm excited but need to wind down before bed",
        "Feeling a bit sad and need cheering up"
    ]

BEGIN
    defaultInput ← DEFAULT_INPUTS[iteration - 1] OR DEFAULT_INPUTS[0]

    prompt ← CreateInquirerPrompt({
        type: "input",
        name: "emotionalText",
        message: "How are you feeling right now?",
        default: defaultInput,
        validate: function(input)
            IF input.trim().length == 0 THEN
                RETURN "Please describe your emotional state"
            END IF
            IF input.length < 10 THEN
                RETURN "Please provide more detail (at least 10 characters)"
            END IF
            RETURN true
        end function,
        transformer: function(input)
            // Show character count as user types
            RETURN input + ColorText(" (" + ToString(input.length) + " chars)", "gray")
        end function
    })

    answer ← AWAIT InquirerPrompt(prompt)
    RETURN answer.emotionalText
END
```

### 2. Content Selection Prompt

```
ALGORITHM: PromptContentSelection
INPUT: recommendations (array of EmotionalRecommendation)
OUTPUT: Promise<string> (content ID)

BEGIN
    // Create choices for Inquirer
    choices ← []

    FOR i ← 0 TO recommendations.length - 1 DO
        rec ← recommendations[i]

        // Format choice display
        rank ← ToString(i + 1) + "."
        title ← rec.title
        qValue ← "(Q: " + FormatNumber(rec.qValue, 2) + ")"

        choiceName ← rank + " " + title + " " +
                     ColorText(qValue, GetQValueColor(rec.qValue))

        choices.append({
            name: choiceName,
            value: rec.contentId,
            short: title
        })
    END FOR

    prompt ← CreateInquirerPrompt({
        type: "list",
        name: "contentId",
        message: "Select content to view:",
        choices: choices,
        pageSize: 7
    })

    answer ← AWAIT InquirerPrompt(prompt)
    RETURN answer.contentId
END
```

### 3. Post-Viewing Feedback Prompt

```
ALGORITHM: PromptPostViewingFeedback
INPUT: none
OUTPUT: Promise<FeedbackInput>

BEGIN
    feedback ← {}

    // Text feedback
    textPrompt ← CreateInquirerPrompt({
        type: "input",
        name: "postText",
        message: "How do you feel after viewing?",
        default: "I feel more relaxed and calm now",
        validate: function(input)
            IF input.trim().length == 0 THEN
                RETURN "Please describe your current state"
            END IF
            RETURN true
        end function
    })

    textAnswer ← AWAIT InquirerPrompt(textPrompt)
    feedback.postText ← textAnswer.postText

    // Optional rating
    ratingPrompt ← CreateInquirerPrompt({
        type: "list",
        name: "rating",
        message: "Rate your experience:",
        choices: [
            { name: "⭐⭐⭐⭐⭐ Excellent", value: 5 },
            { name: "⭐⭐⭐⭐ Good", value: 4 },
            { name: "⭐⭐⭐ Okay", value: 3 },
            { name: "⭐⭐ Poor", value: 2 },
            { name: "⭐ Very Poor", value: 1 }
        ],
        default: 0 // Excellent
    })

    ratingAnswer ← AWAIT InquirerPrompt(ratingPrompt)
    feedback.rating ← ratingAnswer.rating

    RETURN feedback
END
```

### 4. Continue Prompt

```
ALGORITHM: PromptContinue
INPUT: none
OUTPUT: Promise<boolean>

BEGIN
    prompt ← CreateInquirerPrompt({
        type: "confirm",
        name: "continue",
        message: "Try another recommendation to see learning in action?",
        default: true
    })

    answer ← AWAIT InquirerPrompt(prompt)
    RETURN answer.continue
END
```

---

## Visualization Helpers

### 1. Progress Bar Creator

```
ALGORITHM: CreateProgressBar
INPUT: value (float), min (float), max (float), width (integer)
OUTPUT: string

CONSTANTS:
    FILLED_CHAR = "█"
    EMPTY_CHAR = "░"

BEGIN
    // Normalize value to 0-1 range
    normalized ← (value - min) / (max - min)
    normalized ← Clamp(normalized, 0, 1)

    // Calculate filled portion
    filledWidth ← Round(normalized * width)
    emptyWidth ← width - filledWidth

    // Create bar
    bar ← Repeat(FILLED_CHAR, filledWidth) +
          Repeat(EMPTY_CHAR, emptyWidth)

    RETURN bar
END
```

### 2. Table Row Creator

```
ALGORITHM: CreateTableRow
INPUT: cells (array of strings), widths (array of integers), style (string)
OUTPUT: string

BEGIN
    formattedCells ← []

    FOR i ← 0 TO cells.length - 1 DO
        cell ← cells[i]
        width ← widths[i]

        // Pad to width
        paddedCell ← PadRight(cell, width)

        // Apply style
        IF style == "bold" THEN
            paddedCell ← ColorText(paddedCell, "bold")
        END IF

        formattedCells.append(paddedCell)
    END FOR

    row ← "│ " + Join(formattedCells, " │ ") + " │"
    RETURN row
END

SUBROUTINE: CreateTableSeparator
INPUT: widths (array of integers)
OUTPUT: string

BEGIN
    parts ← []

    FOR EACH width IN widths DO
        parts.append(Repeat("─", width))
    END FOR

    separator ← "├─" + Join(parts, "─┼─") + "─┤"
    RETURN separator
END
```

### 3. Section Border Creator

```
ALGORITHM: PrintSectionBorder
INPUT: type (string: "top" or "bottom")
OUTPUT: void

CONSTANTS:
    WIDTH = 70
    TOP_LEFT = "┌"
    TOP_RIGHT = "┐"
    BOTTOM_LEFT = "└"
    BOTTOM_RIGHT = "┘"
    HORIZONTAL = "─"

BEGIN
    IF type == "top" THEN
        border ← TOP_LEFT + Repeat(HORIZONTAL, WIDTH) + TOP_RIGHT
    ELSE
        border ← BOTTOM_LEFT + Repeat(HORIZONTAL, WIDTH) + BOTTOM_RIGHT
    END IF

    Print(ColorText(border, "gray"))
END
```

### 4. Loading Spinner

```
ALGORITHM: DisplayLoadingSpinner
INPUT: message (string)
OUTPUT: void

CONSTANTS:
    FRAMES = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
    FRAME_DELAY = 80 // milliseconds

BEGIN
    spinner ← CreateOraSpinner({
        text: message,
        spinner: {
            frames: FRAMES,
            interval: FRAME_DELAY
        },
        color: "cyan"
    })

    spinner.start()

    // Return spinner for later stopping
    RETURN spinner
END
```

---

## Error Handling

### 1. Demo Error Handler

```
ALGORITHM: HandleDemoError
INPUT: error (Error object)
OUTPUT: void

BEGIN
    ClearTerminal()

    PrintNewline(2)
    Print(ColorText("╔═══════════════════════════════════════════╗", "red"))
    Print(ColorText("║          ⚠️  Demo Error Occurred          ║", "red"))
    Print(ColorText("╚═══════════════════════════════════════════╝", "red"))
    PrintNewline(1)

    Print("Error: " + ColorText(error.message, "red"))
    PrintNewline(1)

    IF error.stack != null THEN
        Print(ColorText("Stack trace:", "gray"))
        Print(ColorText(error.stack, "gray"))
        PrintNewline(1)
    END IF

    // Log to file for debugging
    LogErrorToFile(error)
END
```

### 2. Graceful Recovery

```
ALGORITHM: DisplayErrorRecovery
INPUT: none
OUTPUT: void

BEGIN
    Print("The demo encountered an unexpected issue.")
    Print("This has been logged for debugging.")
    PrintNewline(1)

    retryPrompt ← CreateInquirerPrompt({
        type: "confirm",
        name: "retry",
        message: "Would you like to restart the demo?",
        default: true
    })

    answer ← AWAIT InquirerPrompt(retryPrompt)

    IF answer.retry THEN
        // Restart demo
        RunDemo()
    ELSE
        Print(ColorText("Thank you for your understanding!", "cyan"))
        Exit(0)
    END IF
END
```

### 3. Input Validation

```
ALGORITHM: ValidateEmotionalInput
INPUT: text (string)
OUTPUT: boolean or string (error message)

BEGIN
    // Minimum length check
    IF text.trim().length < 10 THEN
        RETURN "Please provide at least 10 characters"
    END IF

    // Maximum length check
    IF text.length > 500 THEN
        RETURN "Please keep input under 500 characters"
    END IF

    // Check for inappropriate content
    IF ContainsInappropriateContent(text) THEN
        RETURN "Please provide appropriate content"
    END IF

    // Check if it's actually descriptive
    wordCount ← CountWords(text)
    IF wordCount < 3 THEN
        RETURN "Please use at least 3 words to describe your feelings"
    END IF

    RETURN true
END
```

---

## Timing & Performance

### Demo Timing Annotations

```
TIMING ANALYSIS: Complete Demo Flow

Phase 1: Welcome & Introduction
    - Display welcome: 0 seconds (instant)
    - User reads: ~10 seconds
    - Press enter: 2 seconds
    - Total: ~12 seconds

Phase 2: Emotional Input
    - Display prompt: 0 seconds
    - User types/confirms: ~5 seconds
    - Total: ~5 seconds

Phase 3: Emotion Detection
    - Loading spinner: 0.8 seconds
    - Display results: 0 seconds
    - User reads: ~8 seconds
    - Press enter: 2 seconds
    - Total: ~11 seconds

Phase 4: Desired State
    - Loading spinner: 0.6 seconds
    - Display results: 0 seconds
    - User reads: ~6 seconds
    - Press enter: 2 seconds
    - Total: ~9 seconds

Phase 5: Recommendations
    - Loading spinner: 0.7 seconds
    - Display table: 0 seconds
    - User reads: ~10 seconds
    - Select content: ~3 seconds
    - Total: ~14 seconds

Phase 6: Viewing Simulation
    - Progress bar: 2 seconds
    - Display complete: 1 second
    - Total: ~3 seconds

Phase 7: Feedback
    - Feedback prompt: ~5 seconds
    - Rating prompt: ~2 seconds
    - Total: ~7 seconds

Phase 8: Reward Update
    - Loading spinner: 0.5 seconds
    - Display update: 0 seconds
    - User reads: ~8 seconds
    - Press enter: 2 seconds
    - Total: ~11 seconds

Phase 9: Learning Progress
    - Display stats: 0 seconds
    - User reads: ~10 seconds
    - Press enter: 2 seconds
    - Total: ~12 seconds

Phase 10: Continue/End
    - Prompt: ~2 seconds
    - Total: ~2 seconds

TOTAL SINGLE ITERATION: ~86 seconds (~1.4 minutes)
TOTAL THREE ITERATIONS: ~180 seconds (~3 minutes)

Buffer time: ~30 seconds for Q&A
DEMO DURATION: 3.5 minutes
```

### Performance Optimization

```
ALGORITHM: OptimizeDemoPerformance
INPUT: none
OUTPUT: void

BEGIN
    // Pre-load content data
    PreloadContentDatabase()

    // Pre-compute common embeddings
    PrecomputeCommonEmbeddings()

    // Cache Q-values
    WarmUpQLearningCache()

    // Pre-initialize UI components
    InitializeInquirer()
    InitializeChalk()
    InitializeOra()

    // Clear terminal cache
    ClearTerminalBuffer()
END
```

---

## Rehearsal Checklist

### Pre-Demo Setup

```
CHECKLIST: Demo Environment Setup

□ System Check
  □ Node.js version >= 18
  □ All dependencies installed (npm install)
  □ Database seeded with demo content
  □ Environment variables set
  □ Terminal supports Unicode & 256 colors

□ Data Preparation
  □ Demo user created (ID: demo-user-001)
  □ Q-values initialized to 0
  □ Sample content loaded (10+ items)
  □ Embedding vectors pre-computed
  □ Test emotional states prepared

□ Visual Setup
  □ Terminal size: 80x24 or larger
  □ Font supports emojis
  □ Color scheme tested
  □ ASCII art displays correctly
  □ Progress bars render properly

□ Timing Rehearsal
  □ Full demo run-through completed
  □ Timing verified (3 minutes target)
  □ Pause points identified
  □ Narrative prepared
  □ Q&A anticipated

□ Error Handling
  □ Network timeout handling tested
  □ Database connection errors handled
  □ Invalid input recovery verified
  □ Graceful degradation confirmed

□ Backup Plan
  □ Screen recording as backup
  □ Slides with screenshots prepared
  □ Manual fallback narrative ready
  □ Offline mode tested
```

### Demo Script Narrative

```
SCRIPT: Live Demo Narrative

[INTRODUCTION - 15 seconds]
"Welcome to EmotiStream Nexus. This system uses reinforcement learning
to recommend content based on your emotional state. Let me show you
how it works."

[EMOTIONAL INPUT - 5 seconds]
"First, I'll describe my current emotional state. Let's say I'm feeling
stressed after a long day."
[Type/confirm default input]

[EMOTION DETECTION - 10 seconds]
"The system analyzes my emotional state using NLP. Notice how it
detected negative valence, moderate arousal, and high stress. The
primary emotion is sadness."
[Point to values]

[DESIRED STATE - 8 seconds]
"Based on this, it predicts I want to move toward a calm and positive
state. The reasoning is sound - after stress, people typically seek
relaxation."

[RECOMMENDATIONS - 12 seconds]
"Now here's where Q-Learning comes in. These recommendations are ranked
by Q-values - learned estimates of how well each content will help me
reach my desired state. Notice the Q-values start at zero because this
is a fresh model."
[Point to Q-values column]

[SELECTION - 3 seconds]
"I'll select the top recommendation."
[Select content]

[VIEWING - 3 seconds]
"Simulating viewing..."
[Wait for progress bar]

[FEEDBACK - 7 seconds]
"After viewing, I provide feedback describing how I feel now. Let's say
I feel more relaxed and calm."
[Enter feedback and rating]

[REWARD UPDATE - 10 seconds]
"Here's the magic of reinforcement learning. The system calculated a
reward based on my emotional improvement and updated the Q-value.
Watch what happens when I make another query..."

[SECOND ITERATION - 60 seconds]
"Let's try another emotional state. This time, notice how the Q-values
have changed based on what it learned from my first interaction."
[Repeat flow, pointing out Q-value changes]

[LEARNING PROGRESS - 12 seconds]
"This chart shows the learning progress. Each interaction improves the
model's recommendations. Over time, it builds a personalized profile
of what content works best for different emotional states."

[CONCLUSION - 10 seconds]
"That's EmotiStream Nexus - emotion-aware recommendations powered by
reinforcement learning. Thank you!"
```

### Troubleshooting Guide

```
TROUBLESHOOTING: Common Demo Issues

Issue: Spinner doesn't render
Solution: Check terminal supports UTF-8, fallback to dots

Issue: Colors don't display
Solution: Disable colors with --no-color flag

Issue: Slow emotion detection
Solution: Pre-compute embeddings, use cached results

Issue: Database timeout
Solution: Increase timeout, use in-memory fallback

Issue: Table formatting broken
Solution: Check terminal width, reduce table columns

Issue: User input stuck
Solution: Ctrl+C to exit, restart with cached state

Issue: Emoji not rendering
Solution: Use text alternatives: :) for 😊

Issue: Q-values all zero
Solution: Pre-seed with realistic values for demo
```

---

## Color Scheme Reference

```
COLOR SCHEME: EmotiStream Nexus Demo

Primary Colors:
    - Cyan: Headers, titles, system messages
    - White: Default text
    - Gray: Borders, secondary info

Emotional State Colors:
    - Green: Positive valence, low stress
    - Red: Negative valence, high stress
    - Yellow: Moderate/excited arousal
    - Blue: Calm/low arousal
    - Magenta: Confidence indicators

Q-Value Colors:
    - Green: High Q-value (> 0.5)
    - Yellow: Medium Q-value (0.2 - 0.5)
    - White: Low Q-value (0 - 0.2)
    - Gray: Negative Q-value (< 0)

Feedback Colors:
    - Green: Positive reward (> 0.5)
    - Yellow: Moderate reward (0 - 0.5)
    - Red: Negative reward (< 0)

Status Colors:
    - Cyan: Loading/processing
    - Green: Success/completion
    - Yellow: Warning/attention
    - Red: Error/failure
```

---

## Final Notes

### Demo Success Criteria

1. **Technical**
   - All displays render correctly
   - No errors or crashes
   - Timing within 3 minutes
   - Q-learning visible and working

2. **Presentation**
   - Clear emotional state → recommendation flow
   - Learning visible across iterations
   - Professional appearance
   - Smooth transitions

3. **Impact**
   - Judges understand RL application
   - Emotional intelligence demonstrated
   - Practical use case clear
   - Differentiation from competitors

### Edge Cases to Handle

1. Empty recommendations list → Show error message
2. Database connection failure → Use mock data
3. Invalid emotional input → Graceful re-prompt
4. Terminal too small → Warn and resize
5. Interrupted demo → Resume from checkpoint

### Post-Demo Metrics

- Track demo completion rate
- Monitor error occurrences
- Log timing per phase
- Collect user feedback
- Measure judge engagement

---

**END OF PSEUDOCODE SPECIFICATION**
