import SwiftUI
import SwiftData

@main
struct VibeCheckApp: App {
    var sharedModelContainer: ModelContainer = {
        let schema = Schema([
            WatchHistory.self,
            UserPreferences.self,
            MoodLog.self,
            WatchlistItem.self,
            MediaInteraction.self
        ])
        let modelConfiguration = ModelConfiguration(schema: schema, isStoredInMemoryOnly: false)

        do {
            return try ModelContainer(for: schema, configurations: [modelConfiguration])
        } catch {
            fatalError("Could not create ModelContainer: \(error)")
        }
    }()

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .modelContainer(sharedModelContainer)
    }
}

// MARK: - App Theme (Moved here for compilation)
struct AppTheme {
    // MARK: - Icon Palette
    // Derived from the "Futuristic Minimal" app icon
    static let iconPurple = Color(red: 0.5, green: 0.0, blue: 1.0) // Deep Electric Purple
    static let iconBlue = Color(red: 0.0, green: 0.4, blue: 1.0)   // Vivid Blue
    static let iconDarkBg = Color(red: 0.05, green: 0.05, blue: 0.1) // Almost Black/Dark Navy
    
    static let accentGradient = LinearGradient(
        colors: [iconPurple, iconBlue],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )
    
    // MARK: - Mood Palettes
    // Overrides for specific moods to align with the "Icon Theme"
    static func colors(for mood: String) -> [Color] {
        switch mood {
        case "comfort":
            return [
                iconPurple.opacity(0.6), .pink.opacity(0.4), iconBlue.opacity(0.5),
                .pink.opacity(0.4), iconPurple.opacity(0.5), .pink.opacity(0.3),
                iconBlue.opacity(0.4), iconPurple.opacity(0.6), .pink.opacity(0.4)
            ]
        case "gentle":
            return [
                iconBlue.opacity(0.4), iconPurple.opacity(0.3), .cyan.opacity(0.3),
                iconPurple.opacity(0.3), iconBlue.opacity(0.3), .indigo.opacity(0.2),
                .cyan.opacity(0.2), iconBlue.opacity(0.4), iconPurple.opacity(0.3)
            ]
        case "light":
            return [
                .yellow.opacity(0.5), .orange.opacity(0.4), iconPurple.opacity(0.2),
                .orange.opacity(0.3), .yellow.opacity(0.4), .orange.opacity(0.2),
                iconPurple.opacity(0.2), .orange.opacity(0.3), .yellow.opacity(0.5)
            ]
        case "engaging":
            return [
                .teal.opacity(0.5), .green.opacity(0.4), iconBlue.opacity(0.4),
                .green.opacity(0.3), .teal.opacity(0.4), .mint.opacity(0.3),
                iconBlue.opacity(0.3), .teal.opacity(0.4), .green.opacity(0.5)
            ]
        case "exciting":
            return [
                .red.opacity(0.5), iconPurple.opacity(0.5), iconBlue.opacity(0.4),
                iconPurple.opacity(0.5), .red.opacity(0.4), iconPurple.opacity(0.4),
                iconBlue.opacity(0.3), iconPurple.opacity(0.5), .red.opacity(0.5)
            ]
        case "calming":
            return [
                iconBlue.opacity(0.5), .teal.opacity(0.3), iconDarkBg.opacity(0.8),
                .teal.opacity(0.3), iconBlue.opacity(0.4), .cyan.opacity(0.2),
                iconDarkBg.opacity(0.6), .teal.opacity(0.3), iconBlue.opacity(0.5)
            ]
        default: // balanced
            return [
                iconDarkBg.opacity(0.5), iconBlue.opacity(0.2), iconPurple.opacity(0.2),
                iconBlue.opacity(0.2), iconDarkBg.opacity(0.4), .teal.opacity(0.1),
                iconPurple.opacity(0.2), iconBlue.opacity(0.2), iconDarkBg.opacity(0.5)
            ]
        }
    }
}
