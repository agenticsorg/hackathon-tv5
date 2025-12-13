import SwiftUI
import SwiftData

struct WatchlistView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: \WatchlistItem.addedDate, order: .reverse) private var watchlistItems: [WatchlistItem]
    @Query(sort: \WatchHistory.timestamp, order: .reverse) private var watchHistory: [WatchHistory]

    @State private var selectedTab = 0

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Segmented control
                Picker("View", selection: $selectedTab) {
                    Text("Watchlist").tag(0)
                    Text("History").tag(1)
                }
                .pickerStyle(.segmented)
                .padding()

                if selectedTab == 0 {
                    watchlistContent
                } else {
                    historyContent
                }
            }
            .navigationTitle("My List")
        }
    }

    @ViewBuilder
    private var watchlistContent: some View {
        if watchlistItems.isEmpty {
            ContentUnavailableView(
                "No Saved Items",
                systemImage: "bookmark",
                description: Text("Items you save will appear here")
            )
        } else {
            List {
                ForEach(watchlistItems) { item in
                    WatchlistItemRow(item: item)
                }
                .onDelete(perform: deleteWatchlistItems)
            }
        }
    }

    @ViewBuilder
    private var historyContent: some View {
        if watchHistory.isEmpty {
            ContentUnavailableView(
                "No Watch History",
                systemImage: "clock",
                description: Text("Your viewing history will appear here")
            )
        } else {
            List {
                ForEach(watchHistory) { item in
                    WatchHistoryRow(item: item)
                }
                .onDelete(perform: deleteHistoryItems)
            }
        }
    }

    private func deleteWatchlistItems(at offsets: IndexSet) {
        for index in offsets {
            modelContext.delete(watchlistItems[index])
        }
    }

    private func deleteHistoryItems(at offsets: IndexSet) {
        for index in offsets {
            modelContext.delete(watchHistory[index])
        }
    }
}

struct WatchlistItemRow: View {
    let item: WatchlistItem

    var body: some View {
        HStack(spacing: 12) {
            // Placeholder poster
            RoundedRectangle(cornerRadius: 8)
                .fill(.quaternary)
                .frame(width: 50, height: 75)
                .overlay {
                    Image(systemName: "film")
                        .foregroundStyle(.tertiary)
                }

            VStack(alignment: .leading, spacing: 4) {
                Text(item.mediaTitle)
                    .font(.headline)

                if let platform = item.platform {
                    Text(platform.capitalized)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Text("Added \(item.addedDate, style: .relative)")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }
}

struct WatchHistoryRow: View {
    let item: WatchHistory

    var body: some View {
        HStack(spacing: 12) {
            // Placeholder poster
            RoundedRectangle(cornerRadius: 8)
                .fill(.quaternary)
                .frame(width: 50, height: 75)
                .overlay {
                    Image(systemName: "film")
                        .foregroundStyle(.tertiary)
                }

            VStack(alignment: .leading, spacing: 4) {
                Text(item.mediaTitle)
                    .font(.headline)

                if item.completionPercent > 0 {
                    ProgressView(value: item.completionPercent)
                        .tint(.green)
                }

                HStack {
                    if let moodHint = item.moodHint {
                        Label(moodHint.capitalized, systemImage: "waveform.path.ecg")
                    }

                    Spacer()

                    Text(item.timestamp, style: .relative)
                }
                .font(.caption2)
                .foregroundStyle(.tertiary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }
}

#Preview {
    WatchlistView()
        .modelContainer(for: [WatchlistItem.self, WatchHistory.self], inMemory: true)
}
