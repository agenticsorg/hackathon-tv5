//
// MediaInteractionBar.swift
// VibeCheck
//
// SwiftUI component for thumbs up/down rating and "seen" toggle.
// Integrates with InteractionService for persistence and WASM learning.
//

import SwiftUI
import SwiftData

// MARK: - MediaInteractionBar

/// Compact interaction bar with thumbs up/down and seen toggle
@available(iOS 17.0, *)
struct MediaInteractionBar: View {
    let mediaItem: MediaItem
    let mood: MoodState

    @Environment(\.modelContext) private var modelContext
    @State private var interaction: MediaInteraction?
    @State private var isLoading = false
    @State private var showFeedback = false
    @State private var feedbackMessage = ""

    /// Optional callback when interaction changes
    var onInteractionChanged: ((MediaInteraction) -> Void)?

    var body: some View {
        HStack(spacing: 16) {
            // Thumbs Up
            InteractionButton(
                icon: Rating.thumbsUp.iconName,
                iconOutline: Rating.thumbsUp.iconNameOutline,
                isSelected: interaction?.rating == .thumbsUp,
                selectedColor: .green,
                accessibilityLabel: Rating.thumbsUp.accessibilityLabel
            ) {
                await toggleRating(.thumbsUp)
            }

            // Thumbs Down
            InteractionButton(
                icon: Rating.thumbsDown.iconName,
                iconOutline: Rating.thumbsDown.iconNameOutline,
                isSelected: interaction?.rating == .thumbsDown,
                selectedColor: .red,
                accessibilityLabel: Rating.thumbsDown.accessibilityLabel
            ) {
                await toggleRating(.thumbsDown)
            }

            Spacer()

            // Seen Toggle
            SeenToggleButton(
                hasSeen: interaction?.hasSeen ?? false
            ) {
                await toggleSeen()
            }
        }
        .padding(.horizontal, 4)
        .task {
            await loadInteraction()
        }
        .overlay {
            if showFeedback {
                FeedbackToast(message: feedbackMessage)
                    .transition(.opacity.combined(with: .scale))
            }
        }
        .animation(.spring(duration: 0.3), value: interaction?.rating)
        .animation(.spring(duration: 0.3), value: interaction?.hasSeen)
        .animation(.easeOut(duration: 0.2), value: showFeedback)
    }

    // MARK: - Actions

    private func loadInteraction() async {
        do {
            interaction = try MediaInteraction.find(
                mediaId: mediaItem.id,
                in: modelContext
            )
        } catch {
            print("❌ MediaInteractionBar: Failed to load interaction: \(error)")
        }
    }

    private func toggleRating(_ rating: Rating) async {
        isLoading = true
        defer { isLoading = false }

        do {
            let updatedInteraction = try MediaInteraction.findOrCreate(
                mediaId: mediaItem.id,
                mediaTitle: mediaItem.title,
                in: modelContext
            )

            let previousRating = updatedInteraction.rating
            updatedInteraction.toggleRating(rating)
            updatedInteraction.moodHint = mood.recommendationHint

            try modelContext.save()
            interaction = updatedInteraction

            // Trigger WASM learning
            await triggerLearning(feedback: rating.feedbackType)

            // Show feedback
            if updatedInteraction.rating == nil {
                showFeedbackMessage("Rating cleared")
            } else if updatedInteraction.rating == .thumbsUp {
                showFeedbackMessage("Added to liked")
            } else {
                showFeedbackMessage("Marked as not for you")
            }

            onInteractionChanged?(updatedInteraction)

            // Haptic feedback
            let generator = UIImpactFeedbackGenerator(style: .light)
            generator.impactOccurred()

        } catch {
            print("❌ MediaInteractionBar: Failed to toggle rating: \(error)")
        }
    }

    private func toggleSeen() async {
        isLoading = true
        defer { isLoading = false }

        do {
            let updatedInteraction = try MediaInteraction.findOrCreate(
                mediaId: mediaItem.id,
                mediaTitle: mediaItem.title,
                in: modelContext
            )

            updatedInteraction.toggleSeen()
            updatedInteraction.moodHint = mood.recommendationHint

            // Sync with WatchHistory
            if updatedInteraction.hasSeen {
                try createWatchHistoryEntry()
                await triggerLearning(feedback: .watched)
                showFeedbackMessage("Marked as seen")
            } else {
                try removeWatchHistoryEntry()
                showFeedbackMessage("Removed from seen")
            }

            try modelContext.save()
            interaction = updatedInteraction

            onInteractionChanged?(updatedInteraction)

            // Haptic feedback
            let generator = UIImpactFeedbackGenerator(style: .medium)
            generator.impactOccurred()

        } catch {
            print("❌ MediaInteractionBar: Failed to toggle seen: \(error)")
        }
    }

    private func triggerLearning(feedback: FeedbackType) async {
        // Learning is handled via LearningMemoryService
        // This will be wired up in the parent view or via environment
        if #available(iOS 15.0, *) {
            let learningMemory = LearningMemoryService.shared
            if await learningMemory.isReady {
                do {
                    _ = try await learningMemory.recordFeedback(
                        mood: mood,
                        mediaItem: mediaItem,
                        feedback: feedback
                    )
                } catch {
                    print("⚠️ MediaInteractionBar: Learning failed: \(error)")
                }
            }
        }
    }

    private func createWatchHistoryEntry() throws {
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )

        if try modelContext.fetch(descriptor).isEmpty {
            let entry = WatchHistory(
                mediaId: mediaItem.id,
                mediaTitle: mediaItem.title,
                completionPercent: 1.0,
                moodHint: mood.recommendationHint
            )
            modelContext.insert(entry)
        }
    }

    private func removeWatchHistoryEntry() throws {
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )

        for entry in try modelContext.fetch(descriptor) {
            modelContext.delete(entry)
        }
    }

    private func showFeedbackMessage(_ message: String) {
        feedbackMessage = message
        showFeedback = true

        Task {
            try? await Task.sleep(nanoseconds: 1_500_000_000) // 1.5 seconds
            showFeedback = false
        }
    }
}

// MARK: - InteractionButton

@available(iOS 17.0, *)
private struct InteractionButton: View {
    let icon: String
    let iconOutline: String
    let isSelected: Bool
    let selectedColor: Color
    let accessibilityLabel: String
    let action: () async -> Void

    @State private var isPressed = false

    var body: some View {
        Button {
            Task {
                await action()
            }
        } label: {
            Image(systemName: isSelected ? icon : iconOutline)
                .font(.system(size: 20, weight: .medium))
                .foregroundStyle(isSelected ? selectedColor : .secondary)
                .frame(width: 44, height: 44)
                .background(
                    Circle()
                        .fill(isSelected ? selectedColor.opacity(0.15) : Color.clear)
                )
                .scaleEffect(isPressed ? 0.9 : 1.0)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(accessibilityLabel)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
        .onLongPressGesture(minimumDuration: .infinity, maximumDistance: .infinity) {
            // Never completes
        } onPressingChanged: { pressing in
            withAnimation(.easeInOut(duration: 0.1)) {
                isPressed = pressing
            }
        }
    }
}

// MARK: - SeenToggleButton

@available(iOS 17.0, *)
private struct SeenToggleButton: View {
    let hasSeen: Bool
    let action: () async -> Void

    @State private var isPressed = false

    var body: some View {
        Button {
            Task {
                await action()
            }
        } label: {
            HStack(spacing: 6) {
                Image(systemName: hasSeen ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 16, weight: .medium))

                Text(hasSeen ? "Seen" : "Mark seen")
                    .font(.caption)
                    .fontWeight(.medium)
            }
            .foregroundStyle(hasSeen ? .green : .secondary)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(
                Capsule()
                    .fill(hasSeen ? Color.green.opacity(0.15) : Color.secondary.opacity(0.1))
            )
            .scaleEffect(isPressed ? 0.95 : 1.0)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(hasSeen ? "Marked as seen" : "Mark as seen")
        .accessibilityAddTraits(hasSeen ? .isSelected : [])
        .onLongPressGesture(minimumDuration: .infinity, maximumDistance: .infinity) {
            // Never completes
        } onPressingChanged: { pressing in
            withAnimation(.easeInOut(duration: 0.1)) {
                isPressed = pressing
            }
        }
    }
}

// MARK: - FeedbackToast

private struct FeedbackToast: View {
    let message: String

    var body: some View {
        Text(message)
            .font(.caption)
            .fontWeight(.medium)
            .foregroundStyle(.white)
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(
                Capsule()
                    .fill(Color.black.opacity(0.8))
            )
    }
}

// MARK: - Compact Variant

/// Smaller variant for use in lists or cards
@available(iOS 17.0, *)
struct MediaInteractionBarCompact: View {
    let mediaItem: MediaItem
    let mood: MoodState

    @Environment(\.modelContext) private var modelContext
    @State private var interaction: MediaInteraction?

    var body: some View {
        HStack(spacing: 12) {
            // Thumbs Up (compact)
            Button {
                Task { await toggleRating(.thumbsUp) }
            } label: {
                Image(systemName: interaction?.rating == .thumbsUp
                      ? Rating.thumbsUp.iconName
                      : Rating.thumbsUp.iconNameOutline)
                    .font(.system(size: 16))
                    .foregroundStyle(interaction?.rating == .thumbsUp ? .green : .secondary)
            }
            .buttonStyle(.plain)

            // Thumbs Down (compact)
            Button {
                Task { await toggleRating(.thumbsDown) }
            } label: {
                Image(systemName: interaction?.rating == .thumbsDown
                      ? Rating.thumbsDown.iconName
                      : Rating.thumbsDown.iconNameOutline)
                    .font(.system(size: 16))
                    .foregroundStyle(interaction?.rating == .thumbsDown ? .red : .secondary)
            }
            .buttonStyle(.plain)

            // Seen indicator
            if interaction?.hasSeen ?? false {
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 14))
                    .foregroundStyle(.green)
            }
        }
        .task {
            interaction = try? MediaInteraction.find(mediaId: mediaItem.id, in: modelContext)
        }
    }

    private func toggleRating(_ rating: Rating) async {
        do {
            let updated = try MediaInteraction.findOrCreate(
                mediaId: mediaItem.id,
                mediaTitle: mediaItem.title,
                in: modelContext
            )
            updated.toggleRating(rating)
            updated.moodHint = mood.recommendationHint
            try modelContext.save()
            interaction = updated

            let generator = UIImpactFeedbackGenerator(style: .light)
            generator.impactOccurred()
        } catch {
            print("❌ MediaInteractionBarCompact: Error: \(error)")
        }
    }
}

// MARK: - Preview

@available(iOS 17.0, *)
#Preview("Interaction Bar") {
    VStack(spacing: 20) {
        MediaInteractionBar(
            mediaItem: MediaItem.samples.first!,
            mood: .chill
        )
        .padding()
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12))

        MediaInteractionBarCompact(
            mediaItem: MediaItem.samples[1],
            mood: .chill
        )
        .padding()
    }
    .padding()
    .background(Color(.systemGroupedBackground))
}
