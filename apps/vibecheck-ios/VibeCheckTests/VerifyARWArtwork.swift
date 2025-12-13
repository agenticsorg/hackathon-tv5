import Foundation

@main
struct VerifyARWArtwork {
    static func main() async {
        print("🖼️ Verifying ARW Artwork Fetching...")
        
        do {
            // 1. Perform a real search (requires backend running)
            // We use a query likely to return popular results with artwork
            let results = try await ARWService.shared.search(query: "avatar")
            
            if results.isEmpty {
                 print("⚠️ No results found. Is the backend running?")
                 exit(1)
            }
            
            print("✅ Found \(results.count) results.")
            
            var artworkCount = 0
            
            for item in results {
                print("\n🎬 \(item.title)")
                if let poster = item.posterURL {
                    print("   - Poster: \(poster.absoluteString)")
                    artworkCount += 1
                } else {
                    print("   - Poster: ❌")
                }
                
                if let backdrop = item.backdropURL {
                    print("   - Backdrop: \(backdrop.absoluteString)")
                } else {
                     print("   - Backdrop: ❌")
                }
            }
            
            if artworkCount > 0 {
                print("\n🎉 SUCCESS: Artwork URLs are being parsed correctly!")
                print("   (Note: Ensure your UI displays them via AsyncImage)")
            } else {
                print("\n❌ FAILURE: No artwork URLs found in results.")
                exit(1)
            }
            
        } catch {
            print("❌ Error: \(error)")
            exit(1)
        }
    }
}
