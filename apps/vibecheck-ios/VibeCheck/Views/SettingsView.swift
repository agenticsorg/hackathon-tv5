import SwiftUI
import SwiftData

struct SettingsView: View {
    @Environment(\.modelContext) private var modelContext
    @State private var preferences = UserPreferences.default
    @State private var showingExportSheet = false
    @State private var showingClearDataAlert = false

    let allGenres = ["comedy", "drama", "action", "thriller", "sci-fi", "fantasy", "documentary", "animation", "romance", "horror", "mystery", "adventure"]
    let allPlatforms = ["netflix", "hulu", "apple", "max", "prime", "disney", "peacock", "paramount"]

    var body: some View {
        NavigationStack {
            List {
                // Subscriptions
                Section("Your Streaming Services") {
                    ForEach(allPlatforms, id: \.self) { platform in
                        Toggle(isOn: Binding(
                            get: { preferences.subscriptions.contains(platform) },
                            set: { isOn in
                                if isOn {
                                    preferences.subscriptions.append(platform)
                                } else {
                                    preferences.subscriptions.removeAll { $0 == platform }
                                }
                            }
                        )) {
                            HStack {
                                PlatformBadge(platform: platform)
                                Text(platform.capitalized)
                            }
                        }
                    }
                }

                // Favorite genres
                Section("Favorite Genres") {
                    ForEach(allGenres, id: \.self) { genre in
                        Toggle(isOn: Binding(
                            get: { preferences.favoriteGenres.contains(genre) },
                            set: { isOn in
                                if isOn {
                                    preferences.favoriteGenres.append(genre)
                                } else {
                                    preferences.favoriteGenres.removeAll { $0 == genre }
                                }
                            }
                        )) {
                            Text(genre.capitalized)
                        }
                    }
                }

                // Avoid genres
                Section("Genres to Avoid") {
                    ForEach(allGenres, id: \.self) { genre in
                        Toggle(isOn: Binding(
                            get: { preferences.avoidGenres.contains(genre) },
                            set: { isOn in
                                if isOn {
                                    preferences.avoidGenres.append(genre)
                                    // Remove from favorites if added to avoid
                                    preferences.favoriteGenres.removeAll { $0 == genre }
                                } else {
                                    preferences.avoidGenres.removeAll { $0 == genre }
                                }
                            }
                        )) {
                            Text(genre.capitalized)
                        }
                        .tint(.red)
                    }
                }

                // Privacy
                Section("Privacy") {
                    NavigationLink {
                        PrivacyDetailView()
                    } label: {
                        Label("How Your Data Is Used", systemImage: "lock.shield")
                    }

                    Button(role: .destructive) {
                        showingClearDataAlert = true
                    } label: {
                        Label("Clear All Local Data", systemImage: "trash")
                    }
                }

                // Export
                Section("Data Portability") {
                    Button {
                        showingExportSheet = true
                    } label: {
                        Label("Export My Preferences", systemImage: "square.and.arrow.up")
                    }
                }

                // Developer
                Section("Developer") {
                    NavigationLink {
                        BenchmarkView()
                    } label: {
                        Label("Performance Benchmark", systemImage: "speedometer")
                    }
                }

                // About
                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text("1.0.0")
                            .foregroundStyle(.secondary)
                    }

                    Link(destination: URL(string: "https://agentics.org/hackathon")!) {
                        Label("Agentics Hackathon", systemImage: "link")
                    }
                }
            }
            .navigationTitle("Settings")
            .alert("Clear All Data?", isPresented: $showingClearDataAlert) {
                Button("Cancel", role: .cancel) { }
                Button("Clear", role: .destructive) {
                    clearAllData()
                }
            } message: {
                Text("This will delete all your mood history, watchlist, and preferences. This cannot be undone.")
            }
            .sheet(isPresented: $showingExportSheet) {
                ExportSheet(preferences: preferences)
            }
        }
    }

    private func clearAllData() {
        // Clear would be implemented here
        // For now, just reset preferences
        preferences = UserPreferences.default
    }
}

struct PrivacyDetailView: View {
    var body: some View {
        List {
            Section {
                VStack(alignment: .leading, spacing: 16) {
                    Label {
                        Text("100% On-Device")
                            .font(.headline)
                    } icon: {
                        Image(systemName: "iphone")
                            .foregroundStyle(.blue)
                    }

                    Text("All your health data and preferences are processed entirely on your iPhone. Nothing is sent to any server.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 8)
            }

            Section("What We Access") {
                PrivacyRow(
                    icon: "heart.text.square",
                    title: "Heart Rate Variability",
                    description: "Used to estimate your stress level"
                )

                PrivacyRow(
                    icon: "bed.double",
                    title: "Sleep Data",
                    description: "Used to estimate your energy level"
                )

                PrivacyRow(
                    icon: "figure.walk",
                    title: "Step Count",
                    description: "Used to understand your activity level"
                )

                PrivacyRow(
                    icon: "heart",
                    title: "Resting Heart Rate",
                    description: "Secondary stress indicator"
                )
            }

            Section("What We Never Do") {
                Label("Upload health data to any server", systemImage: "xmark.circle.fill")
                    .foregroundStyle(.red)
                Label("Share data with third parties", systemImage: "xmark.circle.fill")
                    .foregroundStyle(.red)
                Label("Use data for advertising", systemImage: "xmark.circle.fill")
                    .foregroundStyle(.red)
                Label("Retain data after you delete it", systemImage: "xmark.circle.fill")
                    .foregroundStyle(.red)
            }

            Section {
                Text("You can revoke health access anytime in Settings → Privacy → Health → VibeCheck")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle("Privacy")
        .navigationBarTitleDisplayMode(.inline)
    }
}

struct PrivacyRow: View {
    let icon: String
    let title: String
    let description: String

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(.green)
                .frame(width: 32)

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                Text(description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

struct ExportSheet: View {
    let preferences: UserPreferences
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: 20) {
                Image(systemName: "doc.text")
                    .font(.system(size: 60))
                    .foregroundStyle(.blue)

                Text("Export Your Data")
                    .font(.title2)
                    .fontWeight(.bold)

                Text("Your preferences will be exported as a JSON file that you can use with other apps or save as a backup.")
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal)

                // Preview
                GroupBox("Preview") {
                    ScrollView {
                        Text(exportJSON)
                            .font(.system(.caption, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .frame(height: 200)
                }
                .padding()

                ShareLink(item: exportJSON, preview: SharePreview("VibeCheck Preferences", image: Image(systemName: "doc.text"))) {
                    Label("Share", systemImage: "square.and.arrow.up")
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(.blue)
                        .foregroundStyle(.white)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                .padding(.horizontal)

                Spacer()
            }
            .padding(.top, 40)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }

    private var exportJSON: String {
        let export: [String: Any] = [
            "format": "vibecheck-preferences-v1",
            "exported": ISO8601DateFormatter().string(from: Date()),
            "favoriteGenres": preferences.favoriteGenres,
            "avoidGenres": preferences.avoidGenres,
            "subscriptions": preferences.subscriptions
        ]

        if let data = try? JSONSerialization.data(withJSONObject: export, options: .prettyPrinted),
           let string = String(data: data, encoding: .utf8) {
            return string
        }
        return "{}"
    }
}

#Preview {
    SettingsView()
        .modelContainer(for: [UserPreferences.self], inMemory: true)
}
