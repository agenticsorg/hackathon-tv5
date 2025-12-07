# Platform Integration Quick Reference

**For:** Media Gateway Developers
**Updated:** 2025-12-06

---

## 1. Platform API Matrix

| Platform | API Access | Auth Method | Integration Strategy |
|----------|-----------|-------------|---------------------|
| **Netflix** | ❌ No | N/A | Aggregator + Deep Link `netflix://title/{id}` |
| **Prime Video** | ❌ No | N/A | Aggregator + Deep Link `primevideo://detail/{id}` |
| **Disney+** | ❌ No | N/A | Aggregator + Deep Link `disneyplus://content/{id}` |
| **Hulu** | ❌ No | N/A | Aggregator + Deep Link `hulu://watch/{id}` |
| **Apple TV+** | ❌ No | N/A | Aggregator + Deep Link `https://tv.apple.com/show/{id}` |
| **YouTube** | ✅ Yes | OAuth 2.0 | Direct API + `youtube://watch?v={id}` |
| **HBO Max** | ❌ No | N/A | Aggregator + Deep Link `hbomax://feature/{id}` |
| **Peacock** | ❌ No | N/A | Aggregator + Deep Link `peacock://content/{id}` |
| **Paramount+** | ❌ No | N/A | Aggregator + Deep Link `paramountplus://content/{id}` |
| **Crave** | ❌ No | N/A | Aggregator + Deep Link (Canada only) |

**Legend:**
- ✅ Public API available
- ❌ No public API (use aggregator)

---

## 2. Aggregator API Comparison

### Streaming Availability API (Recommended)

```bash
# Endpoint
GET https://streaming-availability.p.rapidapi.com/v2/search/basic

# Headers
X-RapidAPI-Key: YOUR_API_KEY
X-RapidAPI-Host: streaming-availability.p.rapidapi.com

# Example Request
curl "https://streaming-availability.p.rapidapi.com/v2/search/basic?country=us&service=netflix&type=movie&keyword=inception" \
  -H "X-RapidAPI-Key: YOUR_KEY"

# Response
{
  "result": [
    {
      "imdbId": "tt1375666",
      "tmdbId": 27205,
      "title": "Inception",
      "streamingInfo": {
        "us": {
          "netflix": {
            "link": "https://www.netflix.com/title/70131314",
            "leaving": 1718668800,
            "availableSince": 1580515200
          }
        }
      }
    }
  ]
}
```

**Coverage:** 60+ countries, 150+ platforms
**Pricing:** Pay-per-request
**Best For:** Deep links, expiry dates, broad platform coverage

### Watchmode API

```bash
# Endpoint
GET https://api.watchmode.com/v1/search/

# Example Request
curl "https://api.watchmode.com/v1/search/?apiKey=YOUR_KEY&search_field=name&search_value=inception"

# Response
{
  "title_results": [
    {
      "id": 123456,
      "name": "Inception",
      "type": "movie",
      "year": 2010,
      "imdb_id": "tt1375666",
      "tmdb_id": 27205
    }
  ]
}

# Get Streaming Sources
GET https://api.watchmode.com/v1/title/{id}/sources/
```

**Coverage:** 200+ services, 50+ countries
**Pricing:** Tiered subscription
**Best For:** Episode-level data (Tier 1 countries)

---

## 3. OAuth 2.0 Flows

### YouTube OAuth 2.0 (Web/Mobile)

```javascript
// Step 1: Authorization Request
const authUrl = 'https://accounts.google.com/o/oauth2/v2/auth';
const params = new URLSearchParams({
  client_id: 'YOUR_CLIENT_ID',
  redirect_uri: 'https://yourdomain.com/oauth/callback',
  response_type: 'code',
  scope: 'https://www.googleapis.com/auth/youtube.readonly',
  state: generateRandomState(),
  code_challenge: generateCodeChallenge(), // PKCE
  code_challenge_method: 'S256'
});
window.location.href = `${authUrl}?${params}`;

// Step 2: Token Exchange (server-side)
const tokenUrl = 'https://oauth2.googleapis.com/token';
const tokenResponse = await fetch(tokenUrl, {
  method: 'POST',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  body: new URLSearchParams({
    client_id: 'YOUR_CLIENT_ID',
    client_secret: 'YOUR_CLIENT_SECRET', // Optional for public clients
    code: authorizationCode,
    code_verifier: codeVerifier, // PKCE
    grant_type: 'authorization_code',
    redirect_uri: 'https://yourdomain.com/oauth/callback'
  })
});

// Response
{
  "access_token": "ya29.a0AfH6SMB...",
  "refresh_token": "1//0gH6SMBq...",
  "expires_in": 3600,
  "token_type": "Bearer",
  "scope": "https://www.googleapis.com/auth/youtube.readonly"
}
```

### Device Authorization Grant (TV/CLI)

```javascript
// Step 1: Request Device Code
const deviceCodeUrl = 'https://oauth2.googleapis.com/device/code';
const deviceResponse = await fetch(deviceCodeUrl, {
  method: 'POST',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  body: new URLSearchParams({
    client_id: 'YOUR_CLIENT_ID',
    scope: 'https://www.googleapis.com/auth/youtube.readonly'
  })
});

// Response
{
  "device_code": "AH-1Ng3JuZUvMsFw...",
  "user_code": "GQVQ-JKEC",
  "verification_url": "https://www.google.com/device",
  "expires_in": 1800,
  "interval": 5
}

// Step 2: Display to User
console.log(`Go to ${verification_url} and enter code: ${user_code}`);

// Step 3: Poll for Token
async function pollForToken(deviceCode, interval) {
  while (true) {
    await sleep(interval * 1000);

    const tokenResponse = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        client_id: 'YOUR_CLIENT_ID',
        device_code: deviceCode,
        grant_type: 'urn:ietf:params:oauth:grant-type:device_code'
      })
    });

    const data = await tokenResponse.json();

    if (data.access_token) {
      return data; // Success!
    } else if (data.error === 'authorization_pending') {
      continue; // Keep polling
    } else if (data.error === 'slow_down') {
      interval += 5; // Increase polling interval
    } else {
      throw new Error(`Authorization failed: ${data.error}`);
    }
  }
}
```

---

## 4. Deep Link Implementation

### iOS Universal Links

**Setup File:** `/.well-known/apple-app-site-association`

```json
{
  "applinks": {
    "apps": [],
    "details": [
      {
        "appID": "TEAM_ID.com.example.medigateway",
        "paths": [
          "/watch/*",
          "/content/*",
          "/movie/*",
          "/tv/*"
        ]
      }
    ]
  }
}
```

**Hosting Requirements:**
- HTTPS (required)
- No redirects
- No .json extension
- Content-Type: `application/json`
- Accessible at: `https://yourdomain.com/.well-known/apple-app-site-association`

**iOS App Configuration (Xcode):**

```swift
// AppDelegate.swift
func application(_ application: UIApplication,
                 continue userActivity: NSUserActivity,
                 restorationHandler: @escaping ([UIUserActivityRestoring]?) -> Void) -> Bool {

    guard userActivity.activityType == NSUserActivityTypeBrowsingWeb,
          let url = userActivity.webpageURL else {
        return false
    }

    // Handle deep link
    if url.path.starts(with: "/watch/") {
        let contentId = url.lastPathComponent
        navigateToContent(id: contentId)
        return true
    }

    return false
}
```

### Android App Links

**Setup File:** `/.well-known/assetlinks.json`

```json
[
  {
    "relation": ["delegate_permission/common.handle_all_urls"],
    "target": {
      "namespace": "android_app",
      "package_name": "com.example.medigateway",
      "sha256_cert_fingerprints": [
        "AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90"
      ]
    }
  }
]
```

**Android Manifest:**

```xml
<activity android:name=".MainActivity">
    <intent-filter android:autoVerify="true">
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />

        <data android:scheme="https" />
        <data android:host="media-gateway.example.com" />
        <data android:pathPrefix="/watch" />
    </intent-filter>
</activity>
```

**Kotlin Handler:**

```kotlin
override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    intent?.data?.let { uri ->
        if (uri.path?.startsWith("/watch/") == true) {
            val contentId = uri.lastPathSegment
            navigateToContent(contentId)
        }
    }
}
```

---

## 5. Metadata Schema

### Unified MediaContent Type

```typescript
interface MediaContent {
  // Internal
  id: string;                    // UUID
  mediaType: 'movie' | 'tv';

  // Core metadata
  title: string;
  overview: string;
  releaseDate: string;           // ISO 8601
  posterPath: string | null;
  backdropPath: string | null;

  // Ratings
  voteAverage: number;           // 0-10
  voteCount: number;
  popularity: number;

  // Classification
  genreIds: number[];

  // External IDs (cross-reference)
  externalIds?: {
    eidr?: string;
    tmdb?: number;
    imdb?: string;
    gracenote?: string;
  };

  // Platform availability
  availability?: PlatformAvailability[];
}

interface PlatformAvailability {
  platform: string;              // "netflix", "prime", "disney", etc.
  region: string;                // ISO 3166-1 alpha-2 (US, UK, CA, etc.)
  type: 'subscription' | 'rental' | 'purchase' | 'free';
  price?: number;
  currency?: string;             // ISO 4217 (USD, GBP, EUR, etc.)
  deepLink: string;              // Platform-specific deep link
  availableFrom?: string;        // ISO 8601 timestamp
  expiresAt?: string;            // ISO 8601 timestamp (nullable)
}
```

---

## 6. Regional Content Handling

### Geolocation Detection

```typescript
// Server-side (Node.js)
import maxmind from 'maxmind';

const geoReader = await maxmind.open('/path/to/GeoLite2-Country.mmdb');

function getUserCountry(ip: string): string {
  const result = geoReader.get(ip);
  return result?.country?.iso_code || 'US'; // Default to US
}

// API Route
app.get('/api/content', (req, res) => {
  const userIp = req.headers['x-forwarded-for'] || req.connection.remoteAddress;
  const country = getUserCountry(userIp);

  // Query aggregator with country filter
  const results = await streamingAPI.search({
    query: req.query.q,
    country: country
  });

  res.json(results);
});
```

### Currency Formatting

```typescript
function formatPrice(amount: number, currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: currency
  }).format(amount);
}

// Usage
formatPrice(3.99, 'USD', 'en-US'); // "$3.99"
formatPrice(3.99, 'GBP', 'en-GB'); // "£3.99"
formatPrice(3.99, 'EUR', 'de-DE'); // "3,99 €"
```

---

## 7. Privacy Compliance Checklist

### GDPR/CCPA Requirements

- [ ] **Consent Management**
  - [ ] Granular opt-in controls (not all-or-nothing)
  - [ ] Clear, plain language explanations
  - [ ] Easy to change preferences later
  - [ ] No dark patterns

- [ ] **User Data Rights**
  - [ ] Data access endpoint (GET /api/user/data)
  - [ ] Data deletion endpoint (DELETE /api/user/account)
  - [ ] Single-click opt-out (not multi-step cookie preferences)
  - [ ] Data portability (JSON export)

- [ ] **Privacy Policy**
  - [ ] Specific data fields disclosed
  - [ ] Specific purposes stated
  - [ ] Data retention timelines
  - [ ] Third-party disclosures
  - [ ] Easy to find (all pages)

### VPPA Requirements

- [ ] **Video Viewing Data**
  - [ ] Explicit consent (not buried in TOS)
  - [ ] Separate from general consent
  - [ ] Easy opt-out
  - [ ] Applies to third-party sharing

- [ ] **Tracking Pixels**
  - [ ] Avoid Meta Pixel for video tracking
  - [ ] Minimize embedded video players
  - [ ] First-party analytics preferred

---

## 8. Caching Strategy

### Redis/Valkey Caching

```typescript
import Redis from 'ioredis';

const redis = new Redis({
  host: process.env.REDIS_HOST,
  port: 6379,
  password: process.env.REDIS_PASSWORD
});

// Cache content metadata (24 hours)
async function getCachedContent(contentId: string) {
  const cacheKey = `content:${contentId}`;
  const cached = await redis.get(cacheKey);

  if (cached) {
    return JSON.parse(cached);
  }

  // Fetch from aggregator API
  const content = await fetchFromAggregator(contentId);

  // Cache for 24 hours
  await redis.setex(cacheKey, 86400, JSON.stringify(content));

  return content;
}

// Cache platform availability (6 hours)
async function getCachedAvailability(contentId: string, region: string) {
  const cacheKey = `availability:${contentId}:${region}`;
  const cached = await redis.get(cacheKey);

  if (cached) {
    return JSON.parse(cached);
  }

  const availability = await fetchAvailability(contentId, region);

  // Cache for 6 hours
  await redis.setex(cacheKey, 21600, JSON.stringify(availability));

  return availability;
}

// Invalidate cache (manual or webhook)
async function invalidateContent(contentId: string) {
  await redis.del(`content:${contentId}`);

  // Also invalidate availability for all regions
  const keys = await redis.keys(`availability:${contentId}:*`);
  if (keys.length > 0) {
    await redis.del(...keys);
  }
}
```

---

## 9. Error Handling

### Aggregator API Errors

```typescript
async function safeAggregatorCall<T>(
  apiCall: () => Promise<T>,
  fallback: T
): Promise<T> {
  try {
    return await apiCall();
  } catch (error) {
    console.error('Aggregator API error:', error);

    // Return cached data if available
    // Or return fallback (empty results)
    return fallback;
  }
}

// Usage
const results = await safeAggregatorCall(
  () => streamingAPI.search({ query: 'inception' }),
  { results: [], totalPages: 0 }
);
```

### Deep Link Fallback

```typescript
function openPlatformContent(platform: string, contentId: string) {
  const deepLinks = {
    netflix: `netflix://title/${contentId}`,
    prime: `primevideo://detail/${contentId}`,
    disney: `disneyplus://content/${contentId}`,
    hulu: `hulu://watch/${contentId}`,
  };

  const webLinks = {
    netflix: `https://www.netflix.com/title/${contentId}`,
    prime: `https://www.amazon.com/gp/video/detail/${contentId}`,
    disney: `https://www.disneyplus.com/video/${contentId}`,
    hulu: `https://www.hulu.com/watch/${contentId}`,
  };

  const deepLink = deepLinks[platform];
  const webLink = webLinks[platform];

  // Try deep link first
  window.location.href = deepLink;

  // Fallback to web after 2 seconds (if app not installed)
  setTimeout(() => {
    window.location.href = webLink;
  }, 2000);
}
```

---

## 10. Testing Commands

### Test Aggregator API

```bash
# Streaming Availability API
curl "https://streaming-availability.p.rapidapi.com/v2/search/basic?country=us&keyword=inception" \
  -H "X-RapidAPI-Key: YOUR_KEY" \
  -H "X-RapidAPI-Host: streaming-availability.p.rapidapi.com"

# Watchmode API
curl "https://api.watchmode.com/v1/search/?apiKey=YOUR_KEY&search_field=name&search_value=inception"
```

### Test YouTube API

```bash
# Search
curl "https://www.googleapis.com/youtube/v3/search?part=snippet&q=inception&key=YOUR_API_KEY"

# Video details
curl "https://www.googleapis.com/youtube/v3/videos?part=snippet,statistics&id=VIDEO_ID&key=YOUR_API_KEY"
```

### Test Deep Links (iOS Simulator)

```bash
# Open URL in iOS Simulator
xcrun simctl openurl booted "https://media-gateway.example.com/watch/12345"
```

### Test Deep Links (Android)

```bash
# Open URL on Android device/emulator
adb shell am start -W -a android.intent.action.VIEW -d "https://media-gateway.example.com/watch/12345"
```

---

## 11. Environment Variables

```bash
# Aggregator APIs
STREAMING_AVAILABILITY_API_KEY=your_key_here
WATCHMODE_API_KEY=your_key_here

# YouTube
YOUTUBE_CLIENT_ID=your_client_id
YOUTUBE_CLIENT_SECRET=your_client_secret
YOUTUBE_API_KEY=your_api_key

# TMDB (for metadata enrichment)
TMDB_API_KEY=your_tmdb_key
TMDB_ACCESS_TOKEN=your_tmdb_token

# Redis/Valkey
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_password

# Database
DATABASE_URL=postgresql://user:pass@host:5432/medigateway

# Security
JWT_SECRET=your_jwt_secret_min_32_chars
ENCRYPTION_KEY=your_encryption_key_32_chars

# Privacy
CONSENT_MANAGEMENT_KEY=onetrust_or_cookieyes_key

# GCP
GOOGLE_CLOUD_PROJECT=your_project_id
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

---

## 12. Useful Links

### Documentation
- [Streaming Platform Research](/workspaces/media-gateway/docs/STREAMING_PLATFORM_SPECIFICATION.md)
- [Research Summary](/workspaces/media-gateway/docs/STREAMING_PLATFORM_RESEARCH_SUMMARY.md)
- [Architecture Blueprint](/tmp/media-gateway-research/research/FINAL_ARCHITECTURE_BLUEPRINT.md)

### APIs
- [Streaming Availability API](https://www.movieofthenight.com/about/api/)
- [Watchmode API](https://api.watchmode.com/)
- [YouTube Data API](https://developers.google.com/youtube/v3)
- [TMDB API](https://developers.themoviedb.org/3)

### Standards
- [RFC 7636 - PKCE](https://datatracker.ietf.org/doc/html/rfc7636)
- [RFC 8628 - Device Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [RFC 9700 - OAuth 2.0 Security BCP](https://datatracker.ietf.org/doc/html/rfc9700)

### Privacy
- [GDPR Official Site](https://gdpr.eu/)
- [CCPA Overview](https://oag.ca.gov/privacy/ccpa)
- [VPPA Guidance](https://www.onetrust.com/blog/what-the-video-privacy-protection-act-means-for-digital-consent-today/)

---

**Quick Reference Version:** 1.0.0
**Last Updated:** 2025-12-06
**Maintainer:** Media Gateway Engineering Team

---
