import SwiftUI

struct RecommendationCard: View {
    let item: MediaItem
    let mood: MoodState

    @State private var isPressed = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Hero image area
            ZStack(alignment: .bottomLeading) {
                // Poster/backdrop
                ZStack {
                    if let url = item.backdropURL ?? item.posterURL {
                        AsyncImage(url: url) { phase in
                            switch phase {
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                                    .transition(.opacity)
                            default:
                                // Loading or error state: show gradient
                                Rectangle()
                                    .fill(
                                        LinearGradient(
                                            colors: genreColors,
                                            startPoint: .topLeading,
                                            endPoint: .bottomTrailing
                                        )
                                    )
                            }
                        }
                    } else {
                        // No logic fallback
                        Rectangle()
                            .fill(
                                LinearGradient(
                                    colors: genreColors,
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                    }
                }
                .overlay {
                    // Genre icon (only show if no image, or maybe always? 
                    // Let's hide it if we have an image to reduce clutter, or keep it subtle)
                    if item.backdropPath == nil && item.posterPath == nil {
                        Image(systemName: genreIcon)
                            .font(.system(size: 50))
                            .foregroundStyle(.white.opacity(0.3))
                    }
                }

                // Gradient overlay
                LinearGradient(
                    colors: [.clear, .black.opacity(0.8)],
                    startPoint: .center,
                    endPoint: .bottom
                )

                // Mood match badge
                HStack(spacing: 4) {
                    Image(systemName: "waveform.path.ecg")
                        .symbolEffect(.variableColor.iterative.dimInactiveLayers)
                    Text("Matches your vibe")
                        .font(.caption2)
                        .fontWeight(.medium)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(.ultraThinMaterial, in: Capsule())
                .padding(12)
            }
            .frame(height: 160)
            .clipped()

            // Content area
            VStack(alignment: .leading, spacing: 10) {
                // Title
                Text(item.title)
                    .font(.headline)
                    .lineLimit(1)

                // Metadata row
                HStack(spacing: 12) {
                    Label(item.formattedRuntime, systemImage: "clock")
                    Label(String(item.year), systemImage: "calendar")

                    if let rating = item.rating {
                        Label(String(format: "%.1f", rating), systemImage: "star.fill")
                            .foregroundStyle(.yellow)
                    }

                    Spacer()

                    // Platform badges
                    HStack(spacing: -6) {
                        ForEach(item.platforms.prefix(3), id: \.self) { platform in
                            PlatformBadge(platform: platform)
                        }
                    }
                }
                .font(.caption)
                .foregroundStyle(.secondary)

                // Genre pills
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        ForEach(item.genres, id: \.self) { genre in
                            Text(genre.capitalized)
                                .font(.caption2)
                                .fontWeight(.medium)
                                .padding(.horizontal, 10)
                                .padding(.vertical, 5)
                                .background(.quaternary, in: Capsule())
                        }
                    }
                }
                
                // Sommelier Rationale
                if let rationale = item.sommelierRationale {
                    HStack(alignment: .top, spacing: 6) {
                        Image(systemName: "sparkles.rectangle.stack.fill") // Or a better "Sommelier" icon
                            .foregroundStyle(.purple)
                            .font(.caption)
                            .padding(.top, 2)
                        
                        Text(rationale)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .lineLimit(2)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(.top, 4)
                }
            }
            .padding()
        }
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
        .shadow(color: .black.opacity(0.1), radius: 10, y: 5)
        .scaleEffect(isPressed ? 0.97 : 1)
        .animation(.spring(duration: 0.2), value: isPressed)
        .onLongPressGesture(minimumDuration: .infinity, maximumDistance: .infinity) {
            // Never completes
        } onPressingChanged: { pressing in
            isPressed = pressing
        }
    }

    private var genreColors: [Color] {
        guard let primaryGenre = item.genres.first?.lowercased() else {
            return [.gray, .gray.opacity(0.7)]
        }

        switch primaryGenre {
        case "comedy":
            return [.yellow, .orange]
        case "drama":
            return [.purple, .indigo]
        case "action", "adventure":
            return [.red, .orange]
        case "thriller", "horror":
            return [.gray, .black]
        case "sci-fi", "fantasy":
            return [.blue, .purple]
        case "documentary":
            return [.green, .teal]
        case "animation":
            return [.pink, .purple]
        case "romance":
            return [.pink, .red]
        case "mystery":
            return [.indigo, .black]
        default:
            return [.blue, .cyan]
        }
    }

    private var genreIcon: String {
        guard let primaryGenre = item.genres.first?.lowercased() else {
            return "film"
        }

        switch primaryGenre {
        case "comedy":
            return "face.smiling"
        case "drama":
            return "theatermasks"
        case "action":
            return "bolt.fill"
        case "adventure":
            return "map"
        case "thriller", "mystery":
            return "magnifyingglass"
        case "horror":
            return "moon.stars"
        case "sci-fi":
            return "sparkles"
        case "fantasy":
            return "wand.and.stars"
        case "documentary":
            return "globe"
        case "animation":
            return "paintpalette"
        case "romance":
            return "heart.fill"
        case "history":
            return "scroll"
        default:
            return "film"
        }
    }
}

struct PlatformBadge: View {
    let platform: String

    var body: some View {
        Circle()
            .fill(platformColor)
            .frame(width: 26, height: 26)
            .overlay {
                Text(platform.prefix(1).uppercased())
                    .font(.system(size: 11, weight: .bold, design: .rounded))
                    .foregroundStyle(.white)
            }
            .overlay {
                Circle()
                    .stroke(Color(.systemBackground), lineWidth: 2)
            }
    }

    private var platformColor: Color {
        switch platform.lowercased() {
        case "netflix":
            return .red
        case "hulu":
            return .green
        case "disney+", "disney":
            return .blue
        case "max", "hbo":
            return .purple
        case "prime", "amazon":
            return .cyan
        case "apple", "apple tv+":
            return .gray
        case "peacock":
            return .yellow
        case "paramount+", "paramount":
            return .blue
        case "discovery+":
            return .blue
        case "fx":
            return .orange
        default:
            return .secondary
        }
    }
}

#Preview {
    ScrollView {
        VStack(spacing: 20) {
            ForEach(MediaItem.samples.prefix(3)) { item in
                RecommendationCard(item: item, mood: .chill)
            }
        }
        .padding()
    }
}
