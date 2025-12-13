import SwiftUI

struct ContentView: View {
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            ForYouView()
                .tabItem {
                    Label("For You", systemImage: "sparkles.rectangle.stack")
                }
                .tag(0)

            VibeCheckView()
                .tabItem {
                    Label("Vibe Check", systemImage: "waveform.path.ecg")
                }
                .tag(1)

            WatchlistView()
                .tabItem {
                    Label("Watchlist", systemImage: "bookmark.fill")
                }
                .tag(2)

            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gear")
                }
                .tag(3)
        }
        .tint(.primary)
    }
}

#Preview {
    ContentView()
}
