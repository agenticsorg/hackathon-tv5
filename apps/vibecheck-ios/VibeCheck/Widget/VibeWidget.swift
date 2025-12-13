import WidgetKit
import SwiftUI

// MARK: - Widget Entry

struct VibeEntry: TimelineEntry {
    let date: Date
    let moodHint: String
    let moodIcon: String
    let moodDescription: String
    let topRecommendation: String?
    let platform: String?
    let confidence: Double
}

// MARK: - Timeline Provider

struct VibeProvider: TimelineProvider {
    func placeholder(in context: Context) -> VibeEntry {
        VibeEntry(
            date: Date(),
            moodHint: "balanced",
            moodIcon: "circle.grid.cross.fill",
            moodDescription: "Balanced",
            topRecommendation: "Abbott Elementary",
            platform: "Hulu",
            confidence: 0.5
        )
    }

    func getSnapshot(in context: Context, completion: @escaping (VibeEntry) -> Void) {
        let entry = VibeEntry(
            date: Date(),
            moodHint: "chill",
            moodIcon: "leaf.circle.fill",
            moodDescription: "Chill",
            topRecommendation: "Ted Lasso",
            platform: "Apple TV+",
            confidence: 0.8
        )
        completion(entry)
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<VibeEntry>) -> Void) {
        // In a real implementation, this would read from shared UserDefaults/AppGroup
        // For now, return a static timeline that updates hourly
        let currentDate = Date()

        let entry = VibeEntry(
            date: currentDate,
            moodHint: "engaging",
            moodIcon: "sparkles",
            moodDescription: "Engaged",
            topRecommendation: "Severance",
            platform: "Apple TV+",
            confidence: 0.7
        )

        // Update every hour
        let nextUpdate = Calendar.current.date(byAdding: .hour, value: 1, to: currentDate)!
        let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
        completion(timeline)
    }
}

// MARK: - Widget Views

struct VibeWidgetEntryView: View {
    var entry: VibeProvider.Entry
    @Environment(\.widgetFamily) var family

    var body: some View {
        switch family {
        case .systemSmall:
            SmallWidgetView(entry: entry)
        case .systemMedium:
            MediumWidgetView(entry: entry)
        case .accessoryRectangular:
            RectangularWidgetView(entry: entry)
        case .accessoryCircular:
            CircularWidgetView(entry: entry)
        default:
            SmallWidgetView(entry: entry)
        }
    }
}

struct SmallWidgetView: View {
    let entry: VibeEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Mood indicator
            HStack(spacing: 6) {
                Image(systemName: entry.moodIcon)
                    .font(.title2)
                    .symbolRenderingMode(.hierarchical)
                    .foregroundStyle(moodColor)

                Text(entry.moodDescription)
                    .font(.headline)
                    .fontWeight(.semibold)
            }

            Spacer()

            // Recommendation
            if let title = entry.topRecommendation {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Watch")
                        .font(.caption2)
                        .foregroundStyle(.secondary)

                    Text(title)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .lineLimit(2)

                    if let platform = entry.platform {
                        Text(platform)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        .padding()
    }

    private var moodColor: Color {
        switch entry.moodHint {
        case "comfort": return .purple
        case "gentle": return .blue
        case "light": return .yellow
        case "engaging": return .green
        case "exciting": return .orange
        case "calming": return .teal
        default: return .gray
        }
    }
}

struct MediumWidgetView: View {
    let entry: VibeEntry

    var body: some View {
        HStack(spacing: 16) {
            // Left side - Mood
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 6) {
                    Image(systemName: entry.moodIcon)
                        .font(.title)
                        .symbolRenderingMode(.hierarchical)
                        .foregroundStyle(moodColor)

                    VStack(alignment: .leading) {
                        Text("Your Vibe")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text(entry.moodDescription)
                            .font(.headline)
                    }
                }

                Text("\(entry.moodHint.capitalized) mode")
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(moodColor.opacity(0.2), in: Capsule())
            }

            Divider()

            // Right side - Recommendation
            VStack(alignment: .leading, spacing: 4) {
                Text("Recommended")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let title = entry.topRecommendation {
                    Text(title)
                        .font(.headline)
                        .lineLimit(2)

                    if let platform = entry.platform {
                        HStack(spacing: 4) {
                            Circle()
                                .fill(platformColor(platform))
                                .frame(width: 8, height: 8)
                            Text(platform)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                Spacer()
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding()
    }

    private var moodColor: Color {
        switch entry.moodHint {
        case "comfort": return .purple
        case "gentle": return .blue
        case "light": return .yellow
        case "engaging": return .green
        case "exciting": return .orange
        case "calming": return .teal
        default: return .gray
        }
    }

    private func platformColor(_ platform: String) -> Color {
        switch platform.lowercased() {
        case "netflix": return .red
        case "hulu": return .green
        case "apple tv+", "apple": return .gray
        case "max", "hbo": return .purple
        case "prime": return .cyan
        case "disney+": return .blue
        default: return .secondary
        }
    }
}

struct RectangularWidgetView: View {
    let entry: VibeEntry

    var body: some View {
        HStack {
            Image(systemName: entry.moodIcon)
                .font(.title2)

            VStack(alignment: .leading) {
                Text(entry.moodDescription)
                    .font(.headline)

                if let title = entry.topRecommendation {
                    Text(title)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()
        }
    }
}

struct CircularWidgetView: View {
    let entry: VibeEntry

    var body: some View {
        ZStack {
            Circle()
                .stroke(lineWidth: 3)
                .opacity(0.2)

            Circle()
                .trim(from: 0, to: entry.confidence)
                .stroke(style: StrokeStyle(lineWidth: 3, lineCap: .round))
                .rotationEffect(.degrees(-90))

            Image(systemName: entry.moodIcon)
                .font(.title3)
        }
    }
}

// MARK: - Widget Configuration

struct VibeWidget: Widget {
    let kind: String = "VibeWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: VibeProvider()) { entry in
            VibeWidgetEntryView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName("Your Vibe")
        .description("See your current mood and a personalized recommendation.")
        .supportedFamilies([
            .systemSmall,
            .systemMedium,
            .accessoryRectangular,
            .accessoryCircular
        ])
    }
}

// MARK: - Previews

#Preview(as: .systemSmall) {
    VibeWidget()
} timeline: {
    VibeEntry(
        date: Date(),
        moodHint: "chill",
        moodIcon: "leaf.circle.fill",
        moodDescription: "Chill",
        topRecommendation: "Ted Lasso",
        platform: "Apple TV+",
        confidence: 0.8
    )
}

#Preview(as: .systemMedium) {
    VibeWidget()
} timeline: {
    VibeEntry(
        date: Date(),
        moodHint: "engaging",
        moodIcon: "sparkles",
        moodDescription: "Engaged",
        topRecommendation: "Severance",
        platform: "Apple TV+",
        confidence: 0.7
    )
}
