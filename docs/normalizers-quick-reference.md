# Platform Normalizers Quick Reference

## All Platform Normalizers (9 Platforms)

| Platform | Module | Platform ID | Subscription Tiers | Original Flag |
|----------|--------|-------------|-------------------|---------------|
| Netflix | `netflix` | `netflix` | subscription | - |
| Prime Video | `prime_video` | `prime_video` | subscription, rental, purchase | - |
| Disney+ | `disney_plus` | `disney_plus` | subscription | - |
| HBO Max | `hbo_max` | `hbo_max` | ad-supported, ad-free | `max_original` |
| Hulu | `hulu` | `hulu` | ad-supported, ad-free, live-tv | `hulu_original` |
| Apple TV+ | `apple_tv_plus` | `apple_tv_plus` | premium | `apple_original` |
| Paramount+ | `paramount_plus` | `paramount_plus` | essential, premium, showtime | `paramount_original` |
| Peacock | `peacock` | `peacock` | free, premium, premium-plus | `peacock_original` |
| YouTube | `youtube` | `youtube` | free, rental, purchase | - |

## Deep Link Patterns

### Hulu
- Mobile: `hulu://watch/{content_id}`
- Web: `https://www.hulu.com/watch/{content_id}`
- TV: `hulu://watch/{content_id}`

### Apple TV+
- Mobile: `videos://watch/{content_id}`
- Web: `https://tv.apple.com/us/video/{content_id}`
- TV: `com.apple.tv://watch/{content_id}`

### Paramount+
- Mobile: `paramountplus://content/{content_id}`
- Web: `https://www.paramountplus.com/movies/{content_id}`
- TV: `paramountplus://content/{content_id}`

### Peacock
- Mobile: `peacock://watch/{content_id}`
- Web: `https://www.peacocktv.com/watch/{content_id}`
- TV: `peacock://watch/{content_id}`

## Platform-Specific Genres

### Hulu
- Hulu Originals → Drama
- FX Originals → Drama
- Anime → Animation
- Live TV → Reality

### Apple TV+
- Apple Originals → Drama
- Masterclass → Documentary
- Nature/Wildlife → Documentary

### Paramount+
- Paramount+ Original → Drama
- CBS Originals → Drama
- MTV Originals → Reality
- Nickelodeon → Family + Animation
- Star Trek → Science Fiction
- Showtime → Drama

### Peacock
- Peacock Originals → Drama
- NBC Originals → Drama
- WWE/Wrestling → Sports
- Premier League → Sports
- True Crime → Crime + Documentary

## Standard Genre Mappings (All Platforms)

- Action & Adventure → Action
- Sci-Fi → Science Fiction
- Comedy → Comedy
- Drama → Drama
- Horror → Horror
- Documentary → Documentary
- Animation → Animation
- Family/Kids → Family
- Romance → Romance
- Thriller → Thriller
- Mystery → Mystery
- Crime → Crime
- War → War
- Western → Western
- Music/Musical → Music
- History/Historical → History

## Usage Example

```rust
use ingestion::normalizer::{
    hulu::HuluNormalizer,
    apple_tv_plus::AppleTvPlusNormalizer,
    paramount_plus::ParamountPlusNormalizer,
    peacock::PeacockNormalizer,
};

// Initialize
let hulu = HuluNormalizer::new("api_key".to_string());

// Fetch catalog delta
let since = chrono::Utc::now() - chrono::Duration::days(7);
let raw_content = hulu.fetch_catalog_delta(since, "us").await?;

// Normalize
for raw in raw_content {
    let canonical = hulu.normalize(raw)?;
    println!("Title: {}", canonical.title);
    println!("Platform: {}", canonical.platform_id);
    println!("Tier: {:?}", canonical.external_ids.get("subscription_tier"));
}

// Generate deep link
let deep_link = hulu.generate_deep_link("content123");
println!("Mobile: {:?}", deep_link.mobile_url);
println!("Web: {}", deep_link.web_url);
```

## Testing

Run all normalizer tests:
```bash
cargo test --package ingestion --lib normalizer
```

Run integration tests:
```bash
cargo test --test normalizers_integration_test
```

Run specific platform tests:
```bash
cargo test --package ingestion hulu::tests
cargo test --package ingestion apple_tv_plus::tests
cargo test --package ingestion paramount_plus::tests
cargo test --package ingestion peacock::tests
```

## Rate Limits

All normalizers use consistent rate limits:
- **Max Requests:** 100 per window
- **Window:** 60 seconds
- **API Key Rotation:** Supported via `api_keys` field

## External IDs

All normalizers extract:
- **IMDb ID:** `imdbId` → `external_ids["imdb"]`
- **TMDb ID:** `tmdbId` → `external_ids["tmdb"]`
- **EIDR:** `eidr` → `external_ids["eidr"]`

Plus platform-specific IDs:
- Subscription tier → `external_ids["subscription_tier"]`
- Original flag → `external_ids["{platform}_original"]`
