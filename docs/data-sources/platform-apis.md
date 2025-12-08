# Streaming Platform API Data Sources

## Executive Summary

**Last Updated:** December 5, 2025

This document provides a comprehensive analysis of data sources for media content aggregation across major streaming platforms. **Critical Finding:** Major streaming platforms (Netflix, Amazon Prime, Disney+, Hulu, HBO Max) do NOT provide official public APIs for third-party developers. All streaming availability data must be obtained through third-party aggregation services or web scraping (with legal constraints).

## Table of Contents

1. [Official Streaming Platform APIs](#official-streaming-platform-apis)
2. [Third-Party Aggregation Services](#third-party-aggregation-services)
3. [Metadata Services](#metadata-services)
4. [API Comparison Matrix](#api-comparison-matrix)
5. [Rate Limits & Pricing](#rate-limits--pricing)
6. [Authentication Requirements](#authentication-requirements)
7. [Data Quality & Update Frequency](#data-quality--update-frequency)
8. [Regional Content Variations](#regional-content-variations)
9. [Web Scraping Considerations](#web-scraping-considerations)
10. [Coverage Gaps & Limitations](#coverage-gaps--limitations)
11. [Integration Strategy Recommendations](#integration-strategy-recommendations)

---

## 1. Official Streaming Platform APIs

### Status: NOT AVAILABLE

**Major streaming platforms do NOT provide official public APIs:**

- ❌ **Netflix** - No public API available
- ❌ **Amazon Prime Video** - No public API available
- ❌ **Disney+** - No public API available
- ❌ **Hulu** - No public API available
- ❌ **HBO Max** - No public API available
- ❌ **Apple TV+** - No public API available
- ❌ **Peacock** - No public API available

### Historical Context

Major streaming platforms previously offered APIs (notably Netflix Developer API discontinued in 2014) but have since closed public access to protect competitive advantages and content licensing agreements.

### Implication

**All streaming availability data must be sourced from third-party aggregation services** that collect, normalize, and distribute streaming catalog information through their own APIs.

---

## 2. Third-Party Aggregation Services

### 2.1 Streaming Availability API

**Provider:** Movie of the Night
**Website:** https://www.movieofthenight.com/about/api/
**GitHub:** https://github.com/movieofthenight/ts-streaming-availability

#### Coverage
- **Platforms:** Netflix, Amazon Prime, Disney+, Max (HBO Max), Apple TV, Hulu, and more
- **Countries:** 60+ countries with region-specific availability
- **Content Types:** Movies, TV series, seasons, episodes

#### Key Features
- Streaming availability queries by title or platform
- Deep links to content on streaming platforms
- Expiry dates for time-limited content
- Video quality information
- Available audio tracks and subtitles
- Images, genres, cast information
- Top 10 rankings by platform
- Recently Added and Upcoming lists
- IMDb and TMDB ID mapping

#### API Endpoint
Available via RapidAPI: https://rapidapi.com/movie-of-the-night-movie-of-the-night-default/api/streaming-availability

#### Data Quality
- Real-time availability tracking
- Comprehensive metadata integration
- Supports TypeScript SDK

#### Use Cases
- "Where can I watch this movie?"
- Multi-platform content discovery
- Availability expiration tracking
- Regional content comparison

---

### 2.2 Watchmode API

**Provider:** Watchmode
**Website:** https://api.watchmode.com/

#### Coverage
- **Platforms:** 200+ streaming services across 50+ countries
- Categorizes Netflix, HBO Max, Amazon Prime, Peacock, Disney+, Hulu, and hundreds of smaller services
- **Content Database:** Extensive movies and TV shows catalog

#### Key Features
- Daily updated CSV file with ID mappings (Watchmode ID, IMDb ID, TMDB ID)
- Search and lookup by multiple ID types
- Deep linking to streaming platforms
- Comprehensive metadata
- Country-specific availability
- "Where to watch" functionality answering streaming location queries

#### ID Mapping
Provides cross-reference between:
- Watchmode internal IDs
- IMDb IDs
- TMDB IDs
- Updated daily for accuracy

#### Data Quality
- Updated multiple times daily
- Strong metadata completeness
- Reliable deep linking

#### Integration Approach
1. Query by title or external ID (TMDB/IMDb)
2. Retrieve Watchmode ID
3. Get streaming availability by country
4. Access deep links for direct platform navigation

---

### 2.3 JustWatch API

**Provider:** JustWatch
**Website:** https://media.justwatch.com/content-insights
**Documentation:** https://apis.justwatch.com/docs/api/

#### Coverage
- **VOD Availability:** 250,000+ movies, 60,000+ TV shows
- **Platforms:** Global and local streaming services
- **Countries:** Comprehensive international coverage
- **Data Refresh:** Every 24 hours minimum

#### Key Features
- Streaming popularity rankings (from millions of daily users)
- Cross-referenced and deduplicated content (IMDb, TMDB, EIDR IDs)
- APIs, custom data exports, and dashboards
- AI and manual verification for accuracy
- Consumer intent, behavior, and consumption data
- Demand data for competitive intelligence

#### Business Model
- Commercial API for partners
- Custom data exports available
- Contact: data-partner@justwatch.com

#### Streaming Service Integration
- Requires data feed from streaming providers
- Updated multiple times daily (typically every 6 hours)
- Widget and API integration options

#### Data Quality
- **High accuracy:** AI + manual cross-referencing
- **Popularity metrics:** Real user engagement data
- **Comprehensive metadata:** Multi-source deduplication

#### Use Cases
- Content popularity tracking
- Competitive intelligence
- Market trend analysis
- Multi-platform availability

---

### 2.4 Reelgood API

**Provider:** Reelgood for Business
**Website:** https://data.reelgood.com/
**API Docs:** https://data.reelgood.com/api-docs/api-documentation/

#### Coverage
- **Update Frequency:** Every 5 minutes (real-time)
- **Platforms:** All major US and international streaming services
- **Data Analysis:** October 2025 data shows content overlap analysis

#### Key Features
- Real-time streaming availability
- AI-powered content metadata
- 3rd party ID mapping (IMDb, EIDR, Wiki)
- Reelgood popularity score
- Deep linking to platforms
- Premium quality images
- Entertainment & celebrity data
- Normalized streaming data with canonical RG ID
- JSON format delivery
- Search & browse functionality

#### Data Quality
- **Update Speed:** 5-minute refresh cycles
- **AI Enhancement:** Machine learning for metadata enrichment
- **Trusted by:** Major tech companies

#### Exclusivity Analysis (2025)
- Netflix leads on exclusive content (0-3% cross-platform overlap)
- Comprehensive overlap metrics between Disney+, Hulu, Prime Video

#### Pricing
- Variable based on:
  - Selected metadata fields
  - Number of streaming services tracked
  - Update frequency requirements
  - Delivery mechanism
- Contact: data.reelgood.com for licensing

#### Integration
- REST API architecture
- JSON responses
- Real-time availability queries

---

### 2.5 Utelly API

**Provider:** Utelly (acquired by Synamedia in 2022)
**Website:** https://www.utelly.com/
**API Access:** https://rapidapi.com/utelly/api/utelly

#### Coverage
- **Platforms:** Netflix, Amazon Prime, iTunes, Now TV, Google Play, BBC iPlayer, All 4, My 5, Rakuten TV, ITV Hub, and 50+ others
- **Metadata Sources:** Gracenote, TiVo, IMDb, Netflix, and 50+ providers
- **Content Types:** Movies, series, TV shows

#### Key Features
- Universal search and discovery APIs
- Real-time metadata aggregation
- AI/ML enrichment modules for sparse datasets
- Established ETL process (clean, deduplicated data)
- Search & recommendation APIs
- Content Management System (CMS)
- Promotion engine

#### Ownership & Status
- Acquired by Synamedia (2022)
- Part of Synamedia Go offering
- Continues operation in 2025 with active reviews and updates

#### Data Processing
- Aggregates data from multiple metadata providers
- ETL pipeline ensures data cleanliness
- AI enhancement for incomplete datasets
- Duplicate detection and removal

#### API Capabilities
- Metadata aggregation
- AI/ML enrichments
- Search & recommendation
- Toolkit for TV & OTT content discovery

#### Use Cases
- Universal content search
- Cross-platform discovery
- Enhanced recommendation systems
- OTT platform integration

---

## 3. Metadata Services

### 3.1 The Movie Database (TMDB)

**Provider:** TMDB
**Website:** https://www.themoviedb.org/
**Developer Portal:** https://developer.themoviedb.org/

#### Coverage
- **Content:** Movies, TV shows, people (actors, directors, crew)
- **Metadata:** Comprehensive (titles, plots, cast, crew, images, ratings)
- **Images:** Posters, backdrops, profile photos
- **Ratings:** Multiple sources including TMDB community ratings

#### Key Features
- Free API for non-commercial use
- Extensive metadata (genres, release dates, runtime, budget, revenue)
- Credits (cast and crew with roles)
- Discovery endpoints (trending, now playing, upcoming, popular)
- Image galleries (multiple resolutions)
- User ratings and reviews
- Video trailers and clips
- Recommendations and similar content
- Multi-language support

#### Rate Limits (2025)
- **Current Limit:** ~40 requests per second per IP address
- **CDN Limit:** Maximum 50 requests/second, 20 connections per IP
- **Historical Note:** Original rate limiting (40 requests/10 seconds) disabled December 2019
- **No tiered limits:** Same limits for all users (developer and commercial)

#### Authentication
- **API Key Required:** Free registration
- **Process:** Register account → Request API key from settings
- **Attribution:** Must credit TMDB as data source for non-commercial use

#### Pricing
- **Free:** Non-commercial use with attribution
- **Commercial:** Contact [email protected] for licensing

#### Important Limitation
**❌ Does NOT provide streaming availability data via API**

TMDB displays JustWatch streaming data on their website, but this data is NOT accessible through TMDB's API. Developers must use TMDB for metadata and combine it with a separate streaming availability API (Watchmode, Streaming Availability API, etc.) using TMDB IDs as cross-reference.

#### Recommended Integration Pattern
```
1. Query TMDB API for movie/TV metadata using title or TMDB ID
2. Extract TMDB ID from response
3. Query streaming availability API (Watchmode, etc.) using TMDB ID
4. Combine metadata + streaming availability for complete information
```

---

### 3.2 Open Movie Database (OMDb)

**Provider:** OMDb
**Website:** https://www.omdbapi.com/

#### Coverage
- **Content:** Movies, TV series
- **Source:** IMDb-based data
- **Ratings:** IMDb, Rotten Tomatoes, Metacritic

#### Key Features
- Search by title, IMDb ID, type, or year
- Plots, genres, release dates
- Multiple rating sources (IMDb, Rotten Tomatoes, Metascore)
- Poster URLs
- Cast and crew information
- Simple RESTful interface

#### Rate Limits
- **Free Tier:** 1,000 calls/day
- **Paid Tier:** Removes rate limit (requires small monthly donation)

#### Pricing
- **Free:** 1,000 requests/day with attribution
- **Commercial:** Monthly donation removes limits
- Transparent, accessible pricing model

#### Authentication
- API key required (free registration)

#### Important Limitation
**❌ Does NOT provide streaming availability information**

OMDb is ideal for basic movie metadata and ratings but does not track where content can be streamed. For streaming data, pair with Watchmode or similar service.

#### Use Cases
- Quick metadata lookups
- Rating aggregation from multiple sources
- Simple poster retrieval
- IMDb-based searches

---

## 4. API Comparison Matrix

| Service | Streaming Availability | Metadata | Update Frequency | Countries | Platforms | ID Mapping | Pricing Model |
|---------|----------------------|----------|-----------------|-----------|-----------|------------|---------------|
| **Streaming Availability API** | ✅ Yes | ✅ Yes | Real-time | 60+ | Major platforms | IMDb, TMDB | Freemium/Paid |
| **Watchmode** | ✅ Yes | ✅ Yes | Daily+ | 50+ | 200+ services | IMDb, TMDB | Freemium/Paid |
| **JustWatch** | ✅ Yes | ✅ Yes | Every 24h | Global | Global/Local | IMDb, TMDB, EIDR | Commercial |
| **Reelgood** | ✅ Yes | ✅ Yes | Every 5 min | International | Major platforms | IMDb, EIDR, Wiki | Commercial |
| **Utelly** | ✅ Yes | ✅ Yes | Real-time | International | 50+ platforms | Multiple | Commercial |
| **TMDB** | ❌ No | ✅ Extensive | Continuous | N/A | N/A | IMDb | Free/Commercial |
| **OMDb** | ❌ No | ✅ Basic | Periodic | N/A | N/A | IMDb | Free/Donation |

### Recommended Combinations

#### Option 1: Comprehensive Coverage
- **TMDB** (metadata) + **Watchmode** (streaming availability)
- Best for: High-volume applications with budget flexibility

#### Option 2: Real-Time Focus
- **TMDB** (metadata) + **Reelgood** (real-time availability)
- Best for: Time-sensitive applications requiring 5-minute updates

#### Option 3: Budget-Conscious
- **OMDb** (basic metadata) + **Streaming Availability API** (availability)
- Best for: Startups and prototypes

#### Option 4: Commercial-Grade
- **JustWatch** (comprehensive) + **TMDB** (metadata enrichment)
- Best for: Enterprise applications with commercial licensing

---

## 5. Rate Limits & Pricing

### 5.1 TMDB (Confirmed - 2025)

#### Rate Limits
- **40 requests/second** per IP address (approximate)
- **50 requests/second** maximum (CDN enforced)
- **20 concurrent connections** per IP (CDN enforced)
- No difference between developer and commercial API keys

#### Pricing
- **Free:** Non-commercial use with attribution
- **Commercial License:** Contact [email protected]

#### Notes
- Original 40 requests/10 seconds limit removed December 2019
- Generous limits for most use cases
- High-volume commercial apps may need direct negotiation

---

### 5.2 OMDb

#### Rate Limits
- **Free Tier:** 1,000 requests/day
- **Paid Tier:** Unlimited (with monthly donation)

#### Pricing
- **Free:** $0 (1,000 calls/day)
- **Unlimited:** Small monthly donation (transparent pricing)

#### Authentication
- API key required for all tiers

---

### 5.3 Third-Party Streaming APIs

**Note:** Specific pricing and rate limits for Watchmode, JustWatch, Reelgood, Utelly, and Streaming Availability API vary and are typically:

- **Enterprise/Commercial:** Contact sales for pricing
- **Freemium Models:** Limited free tier with paid upgrades
- **Custom Pricing:** Based on request volume, features, and data granularity

#### Factors Affecting Pricing:
1. **Request Volume:** Calls per month/day
2. **Data Scope:** Number of streaming platforms tracked
3. **Geographic Coverage:** Countries included
4. **Update Frequency:** Real-time vs. daily updates
5. **Metadata Richness:** Basic vs. comprehensive data
6. **Support Level:** Standard vs. premium support

#### Recommendation
Contact each provider directly for current 2025 pricing:
- **Watchmode:** https://api.watchmode.com/
- **JustWatch:** data-partner@justwatch.com
- **Reelgood:** https://data.reelgood.com/
- **Utelly:** https://www.utelly.com/
- **Streaming Availability API:** https://rapidapi.com/movie-of-the-night-movie-of-the-night-default/api/streaming-availability

---

## 6. Authentication Requirements

### 6.1 API Key-Based Authentication

**Services Using API Keys:**
- TMDB
- OMDb
- Watchmode
- Streaming Availability API (via RapidAPI)

#### Typical Process:
1. Register free account on provider website
2. Navigate to account settings / developer section
3. Generate API key
4. Include API key in HTTP headers or query parameters

#### Example (TMDB):
```http
GET https://api.themoviedb.org/3/movie/550?api_key=YOUR_API_KEY
```

#### Example (RapidAPI):
```http
GET https://streaming-availability.p.rapidapi.com/...
Headers:
  X-RapidAPI-Key: YOUR_RAPIDAPI_KEY
  X-RapidAPI-Host: streaming-availability.p.rapidapi.com
```

---

### 6.2 Commercial/Enterprise Authentication

**Services with Commercial Agreements:**
- JustWatch
- Reelgood
- Utelly

#### Process:
1. Contact sales/partnership teams
2. Negotiate commercial license
3. Receive dedicated API credentials (API keys, OAuth tokens, or custom auth)
4. May include IP whitelisting, custom endpoints, or dedicated support

---

### 6.3 OAuth-Based Authentication

Some enterprise streaming APIs may use OAuth 2.0 for secure authentication:
- Client credentials flow
- Authorization tokens with expiration
- Token refresh mechanisms

---

### 6.4 Security Best Practices

1. **Never expose API keys in client-side code**
2. **Use environment variables** for key storage
3. **Implement server-side proxies** for API requests
4. **Rotate keys regularly** for high-security applications
5. **Monitor usage** to detect unauthorized access
6. **Implement rate limiting** on your application layer

---

## 7. Data Quality & Update Frequency

### 7.1 Real-Time vs. Periodic Updates

| Service | Update Frequency | Data Quality | Verification Method |
|---------|-----------------|--------------|---------------------|
| **Reelgood** | Every 5 minutes | AI-powered | Real-time monitoring |
| **Streaming Availability API** | Real-time | Comprehensive | Platform monitoring |
| **JustWatch** | Every 24 hours (min) | High accuracy | AI + Manual verification |
| **Watchmode** | Daily+ | Reliable | Multiple daily updates |
| **Utelly** | Real-time | Clean ETL | AI/ML enrichment |
| **TMDB** | Continuous (user-contributed) | Community-verified | User moderation |
| **OMDb** | Periodic | IMDb-based | Static snapshots |

---

### 7.2 Data Quality Considerations

#### JustWatch
- **Accuracy:** AI and manual cross-referencing
- **Verification:** Millions of daily user interactions validate data
- **Deduplication:** Multiple source IDs (IMDb, TMDB, EIDR) unified

#### Reelgood
- **Speed:** 5-minute refresh for near real-time accuracy
- **AI Enhancement:** Machine learning fills gaps in metadata
- **Trust:** Used by major tech companies

#### Utelly
- **ETL Process:** Clean, deduplicated data pipeline
- **50+ Sources:** Gracenote, TiVo, IMDb, Netflix, and more
- **AI Enrichment:** Builds on sparse datasets for completeness

#### Streaming Availability API
- **Real-Time Monitoring:** Tracks platform catalogs continuously
- **Deep Links:** Validated for direct platform access
- **Expiry Tracking:** Monitors time-limited content availability

#### TMDB
- **Community-Driven:** Millions of users contribute and verify
- **Moderation:** Active community and staff review
- **Comprehensive:** Extensive metadata from passionate users

---

### 7.3 2025 Real-Time Data Trends

According to 2025 research:
- **30% of all data** is real-time in nature by 2025
- **95% of organizations** invest in real-time analytics
- **Sub-second resolution:** Available in modern streaming APIs (e.g., Chainlink Data Streams)
- **Data Quality Integration:** Automated trust scores, lineage, and cataloging (e.g., Qlik Open Lakehouse)

---

## 8. Regional Content Variations

### 8.1 Geographic Availability Challenges

**Why Regional Variations Exist:**
1. **Licensing Agreements:** Content rights vary by country/region
2. **Geo-Blocking:** Platforms restrict access based on IP location
3. **Local Content:** Region-specific shows and movies
4. **Release Windows:** Staggered releases across markets
5. **Language Availability:** Dubbed/subtitled versions by region

---

### 8.2 API Support for Regional Data

#### Streaming Availability API
- **60+ countries** supported
- Region-specific availability queries
- Country codes in API requests

#### Watchmode
- **50+ countries**
- Country-specific catalog queries
- Local and global streaming services

#### JustWatch
- **Global coverage** with local streaming services
- International platform tracking
- Region-based popularity rankings

#### Reelgood
- **International platforms** tracked
- US-focused but expanding globally
- Regional content overlap analysis

---

### 8.3 Implementation Considerations

1. **User Location Detection:**
   - IP geolocation
   - User-selected country preference
   - VPN detection (if required by licensing)

2. **Multi-Region Queries:**
   - Allow users to compare availability across countries
   - "Available in X countries" displays
   - Travel-friendly features showing content accessibility

3. **Caching Strategy:**
   - Cache regional availability separately
   - TTL based on update frequency of API
   - Invalidate cache on platform catalog changes

4. **Legal Compliance:**
   - Respect geo-blocking restrictions in UI
   - Avoid encouraging VPN usage to circumvent licensing
   - Display accurate regional availability only

---

## 9. Web Scraping Considerations

### 9.1 Legal Landscape (2025)

#### Key Legal Frameworks

**Computer Fraud and Abuse Act (CFAA):**
- **HiQ Labs v. LinkedIn (2022):** Ruled that scraping **public data** is legal under CFAA
- **Implication:** Publicly accessible data (no login required) can be scraped
- **Limitation:** Accessing data behind authentication may violate CFAA

**Copyright Law:**
- **Protected Content:** Copyrighted material (posters, descriptions, videos) cannot be redistributed without permission
- **Fair Use:** Limited exceptions for transformative use, commentary, research

**Terms of Service (TOS):**
- **Violation != Criminal Charges:** Breaking TOS doesn't automatically lead to prosecution
- **Civil Liability:** Platform may block access, send cease-and-desist, or pursue civil action
- **Common TOS Clauses:** Prohibit bots, automated data collection, high-frequency access

---

### 9.2 Platform-Specific Considerations

#### Netflix

**Terms of Service:**
- Does not explicitly permit or ban scraping
- **Logging in and scraping:** Likely violates TOS
- **Public data only:** Episode names, descriptions, cast, ratings accessible without login MAY be scraped

**Legal Risk:**
- Copyright law and CFAA may still apply
- Public data (HiQ precedent) may be defensible
- Netflix has not publicly pursued scrapers aggressively (as of 2025)

**Recommendation:**
- ✅ Use third-party APIs (Watchmode, JustWatch, etc.)
- ⚠️ Scraping Netflix directly: High legal risk, unreliable data, likely TOS violation

---

#### Other Platforms (Amazon Prime, Disney+, Hulu, HBO Max)

**Similar Constraints:**
- No official APIs
- TOS typically prohibit automated access
- Copyright protections on content metadata
- Legal gray area for public data scraping

**Recommendation:**
- ❌ Avoid scraping major platforms directly
- ✅ Use established third-party aggregation services

---

### 9.3 Ethical Considerations

According to ethical guidelines for scraping streaming data:

1. **User Consent:** Ensure scraped data respects user privacy
2. **Platform Impact:** Minimize load on platform servers (rate limiting, respectful scraping)
3. **Data Responsibility:** Do not redistribute copyrighted material
4. **Transparency:** Disclose data sources to end users
5. **Compliance:** Adhere to data protection laws (GDPR, CCPA)

---

### 9.4 Technical Challenges of Web Scraping

1. **Anti-Scraping Mechanisms:**
   - CAPTCHA challenges
   - IP rate limiting and blocking
   - Dynamic JavaScript rendering (requires headless browsers)
   - Session fingerprinting and bot detection

2. **Platform Changes:**
   - Frequent UI/HTML structure changes break scrapers
   - Ongoing maintenance required
   - Unpredictable downtime

3. **Data Consistency:**
   - Inconsistent HTML structures
   - Missing or incomplete data
   - Difficult to normalize across platforms

4. **Legal & Ethical Risks:**
   - TOS violations
   - Potential legal action
   - Reputational damage

---

### 9.5 Recommendations: Avoid Web Scraping

**Instead of scraping, use established APIs:**

| Goal | Recommended Solution |
|------|---------------------|
| Streaming availability | Watchmode, JustWatch, Reelgood, Streaming Availability API |
| Movie/TV metadata | TMDB, OMDb |
| Ratings | TMDB (community), OMDb (IMDb, RT, Metascore) |
| Deep links to platforms | Watchmode, Reelgood, Streaming Availability API |
| Popularity rankings | JustWatch, Reelgood |

**Why APIs are Superior:**
1. ✅ Legal and compliant
2. ✅ Reliable and maintained
3. ✅ Normalized data across platforms
4. ✅ Official support and documentation
5. ✅ Faster and more scalable
6. ✅ No anti-bot measures to circumvent

**Only consider scraping if:**
- No API alternative exists for critical data
- Legal counsel confirms compliance
- Ethical guidelines are strictly followed
- Proper rate limiting and respectful access implemented

---

## 10. Coverage Gaps & Limitations

### 10.1 API Coverage Gaps

#### Missing or Limited Data Points

1. **Subscription Pricing:**
   - Most APIs do NOT provide current subscription costs
   - Manual research or separate pricing APIs required
   - Pricing varies by region, promotional offers, and time

2. **Promotional/Free Trial Information:**
   - Limited tracking of free trials, student discounts, bundle deals
   - Requires manual updates or user-contributed data

3. **Content Expiry Dates:**
   - **Partial Coverage:** Streaming Availability API tracks expiry
   - **Limited Adoption:** Many APIs do not provide "leaving soon" data
   - **Accuracy Issues:** Expiry dates can change with licensing renewals

4. **Episode-Level Availability:**
   - Some APIs provide series-level data only
   - Episode-specific availability (e.g., "Season 2, Episode 5 on Hulu") may be limited

5. **4K/HDR/Dolby Atmos Availability:**
   - **Limited Coverage:** Few APIs track video/audio quality per platform
   - **Streaming Availability API:** Provides video quality and audio/subtitle info

6. **Live TV vs. On-Demand:**
   - Most APIs focus on on-demand content
   - Live TV schedules and sports streaming less commonly covered

7. **Platform-Specific Features:**
   - Download availability (offline viewing)
   - Simultaneous streams allowed
   - Ad-supported vs. ad-free tiers
   - Family/profile management

---

### 10.2 Regional Coverage Limitations

1. **Country Support:**
   - APIs vary from 50-60+ countries
   - Emerging markets may have limited coverage
   - Local/regional streaming platforms may be missing

2. **Language Metadata:**
   - Subtitle and audio track availability not always comprehensive
   - Regional content language descriptions may be incomplete

---

### 10.3 Platform Coverage Gaps

#### Smaller/Regional Platforms
- Focus is on major platforms (Netflix, Prime, Disney+, Hulu, HBO Max, Apple TV)
- Smaller services (Tubi, Crackle, Pluto TV, etc.) may have limited or no coverage
- International regional services may be absent

#### Rental vs. Subscription
- Most APIs prioritize subscription streaming (SVOD)
- Rental platforms (TVOD) like iTunes, Google Play may have partial data
- Purchase-to-own options less commonly tracked

---

### 10.4 Data Accuracy Limitations

1. **Catalog Changes:**
   - Even real-time APIs have latency (Reelgood: 5 min, JustWatch: 24 hours)
   - Licensing changes can occur faster than API updates
   - "Ghost content" (listed but not actually available) possible

2. **Deep Link Validation:**
   - Links may break if platforms change URLs
   - Regional deep links may redirect incorrectly
   - Authentication state impacts link functionality

3. **Metadata Inconsistencies:**
   - Different APIs may have conflicting data (release dates, cast, genres)
   - Source data quality varies (user-contributed vs. official)
   - Deduplication may merge incorrect entries

---

### 10.5 Cost & Access Limitations

1. **Free Tiers:**
   - Limited request volumes (OMDb: 1,000/day, TMDB: generous but rate-limited)
   - May lack advanced features (real-time updates, deep links)

2. **Commercial Licensing:**
   - Pricing opacity for JustWatch, Reelgood, Utelly
   - Custom contracts required
   - May be cost-prohibitive for startups

3. **API Key Approval:**
   - Some services require manual approval
   - Commercial use may be restricted

---

### 10.6 Integration Complexity

1. **Multi-API Strategy Required:**
   - No single API provides all data
   - Must combine TMDB (metadata) + Watchmode (availability)
   - ID mapping and data normalization needed

2. **API Changes:**
   - Providers may change endpoints, response formats, pricing
   - Deprecation of features or entire APIs possible (see Netflix API 2014)
   - Ongoing monitoring and adaptation required

---

## 11. Integration Strategy Recommendations

### 11.1 Recommended Architecture

#### For Comprehensive Coverage:

```
┌─────────────────────────────────────────────────┐
│           Your Application Layer                │
│  (User Interface, Search, Recommendations)      │
└─────────────────┬───────────────────────────────┘
                  │
    ┌─────────────┴─────────────┐
    │                           │
    ▼                           ▼
┌──────────────┐      ┌─────────────────────┐
│ TMDB API     │      │ Watchmode /         │
│ (Metadata)   │◄────►│ Streaming Avail API │
│              │      │ (Availability)      │
└──────────────┘      └─────────────────────┘
    │                           │
    │         ID Mapping        │
    │    (TMDB ID, IMDb ID)     │
    └───────────────────────────┘
```

**Workflow:**
1. **User searches** for "Inception"
2. **Query TMDB** for metadata (plot, cast, posters, ratings) → Get TMDB ID
3. **Query Watchmode** using TMDB ID for streaming availability
4. **Combine results** and display to user with deep links

---

### 11.2 Implementation Steps

#### Phase 1: MVP (Minimum Viable Product)

**Goal:** Basic streaming availability search

**APIs:**
- **TMDB** (free, comprehensive metadata)
- **Streaming Availability API** (real-time availability, RapidAPI free tier)

**Features:**
- Search movies/TV shows by title
- Display metadata (poster, plot, ratings)
- Show where content is streaming (region-based)
- Deep links to platforms

**Timeline:** 2-4 weeks

---

#### Phase 2: Enhanced Features

**Goal:** Multi-platform comparison, real-time updates

**APIs:**
- **TMDB** (metadata)
- **Watchmode** (upgrade for 200+ platforms, daily CSV mapping)
- **OMDb** (ratings from multiple sources)

**Features:**
- Compare availability across 200+ platforms
- Multiple rating sources (IMDb, RT, Metascore)
- Filter by subscription vs. rental
- Regional availability comparison

**Timeline:** 4-8 weeks

---

#### Phase 3: Advanced Intelligence

**Goal:** Popularity tracking, commercial-grade data

**APIs:**
- **TMDB** (metadata)
- **JustWatch** (popularity data, commercial license)
- **Reelgood** (5-minute real-time updates)

**Features:**
- Trending content by platform
- Popularity rankings (real user demand)
- Content leaving soon alerts (expiry tracking)
- Exclusive vs. multi-platform content analysis

**Timeline:** 8-12 weeks

---

### 11.3 Caching Strategy

**Why Caching:**
- Reduce API costs (pay-per-request models)
- Improve response time
- Handle rate limits gracefully
- Offline resilience

**Recommended TTL (Time To Live):**

| Data Type | TTL | Reason |
|-----------|-----|--------|
| Metadata (title, plot, cast) | 7-30 days | Rarely changes |
| Posters/images | 30-90 days | Static once released |
| Streaming availability | 6-24 hours | Changes frequently with licensing |
| Trending/popularity | 1-6 hours | Real-time sensitive |
| Deep links | 24 hours | Platforms may change URLs |

**Cache Invalidation:**
- Manual override for major releases
- Webhook support (if API provides)
- Periodic background refresh jobs

---

### 11.4 Error Handling & Fallbacks

**API Downtime:**
1. **Retry Logic:** Exponential backoff (1s, 2s, 4s, 8s)
2. **Fallback APIs:** If Watchmode fails, fallback to Streaming Availability API
3. **Cached Data:** Serve stale data with warning "Last updated X hours ago"
4. **User Messaging:** "Streaming availability temporarily unavailable"

**Rate Limiting:**
1. **Client-Side Rate Limiting:** Prevent exceeding API quotas
2. **Queue Requests:** Background job processing for batch operations
3. **User Throttling:** Limit search frequency per user (e.g., 10 searches/minute)

**Data Quality Issues:**
1. **User Reporting:** Allow users to flag incorrect availability
2. **Multi-Source Verification:** Cross-check data across multiple APIs
3. **Manual Curation:** High-priority content (new releases) manually verified

---

### 11.5 Cost Management

**Estimated Costs (Illustrative - Contact vendors for 2025 pricing):**

| Tier | Monthly Requests | APIs Used | Estimated Cost |
|------|-----------------|-----------|----------------|
| **Prototype** | < 10,000 | TMDB (free) + Streaming Availability API (free tier) | $0 |
| **Startup** | 10,000 - 100,000 | TMDB (free) + Watchmode (paid) | $100 - $500/month |
| **Growth** | 100,000 - 1M | TMDB (commercial) + Watchmode + JustWatch | $500 - $5,000/month |
| **Enterprise** | 1M+ | JustWatch + Reelgood + TMDB (commercial) | $5,000 - $50,000+/month |

**Cost Optimization:**
1. **Aggressive Caching:** Reduce redundant API calls
2. **User Activity Patterns:** Cache popular searches, on-demand for niche queries
3. **Tiered Access:** Free users see cached data, premium users get real-time
4. **Batch Processing:** Bulk API requests for catalog updates (nightly jobs)

---

### 11.6 Compliance & Legal

**Key Requirements:**

1. **API Terms Compliance:**
   - Attribute data sources (TMDB, JustWatch, etc.) as required
   - Do not violate usage restrictions (e.g., no redistribution of raw data)
   - Monitor API terms for changes

2. **User Privacy:**
   - GDPR compliance (EU users)
   - CCPA compliance (California users)
   - Do not share user search data with third parties without consent

3. **Content Licensing:**
   - Do not host or redistribute copyrighted content (posters, videos, descriptions) without permission
   - Use official poster URLs from APIs (TMDB, etc.) with proper attribution
   - Deep link to platforms (do not embed streams)

4. **Geo-Blocking Respect:**
   - Display regionally accurate availability
   - Do not encourage VPN use to circumvent licensing restrictions
   - Implement IP-based region detection

---

### 11.7 Future-Proofing

**Anticipated Changes (2025-2027):**

1. **API Consolidation:** Expect mergers/acquisitions among aggregation providers (e.g., Synamedia acquiring Utelly)
2. **Increased Costs:** As streaming wars intensify, data aggregation may become more expensive
3. **Official Platform APIs:** Unlikely but possible that platforms release limited partner APIs
4. **AI Enrichment:** More APIs will use AI for metadata completeness, quality improvement
5. **Real-Time Standard:** Real-time updates (< 5 min) may become industry standard

**Mitigation Strategies:**

1. **Multi-API Architecture:** Avoid vendor lock-in, use 2+ streaming availability APIs
2. **Abstraction Layer:** Build internal API abstraction to easily swap providers
3. **Contract Negotiations:** Lock in pricing with long-term contracts (if budget allows)
4. **Community Data:** Contribute to open-source projects (if applicable) for fallback data

---

## 12. Appendix: Quick Reference

### API Contact Information

| Service | Primary Contact | Website |
|---------|----------------|---------|
| **Streaming Availability API** | RapidAPI | https://rapidapi.com/movie-of-the-night-movie-of-the-night-default/api/streaming-availability |
| **Watchmode** | Website Contact | https://api.watchmode.com/ |
| **JustWatch** | data-partner@justwatch.com | https://media.justwatch.com/content-insights |
| **Reelgood** | Website Contact | https://data.reelgood.com/ |
| **Utelly** | Website Contact | https://www.utelly.com/ |
| **TMDB** | N/A (Free registration) | https://developer.themoviedb.org/ |
| **OMDb** | N/A (Free registration) | https://www.omdbapi.com/ |

---

### Key Takeaways

1. ❌ **No official streaming platform APIs exist** (Netflix, Prime, Disney+, Hulu, HBO Max)
2. ✅ **Use third-party aggregation services** (Watchmode, JustWatch, Reelgood, Streaming Availability API)
3. 🎬 **Combine metadata + availability:** TMDB/OMDb (metadata) + Watchmode/Streaming Availability API (availability)
4. 🌍 **Regional support varies:** 50-60+ countries depending on API
5. ⚖️ **Web scraping:** High legal risk, avoid in favor of APIs
6. 💰 **Costs:** Free tiers available, commercial licensing for scale
7. 🔄 **Update frequency:** Real-time (Reelgood: 5 min) to daily (JustWatch: 24h)
8. 🔗 **ID mapping critical:** Use TMDB ID or IMDb ID to link metadata and availability APIs
9. 📊 **Data quality:** AI + manual verification (JustWatch), real-time monitoring (Reelgood), community-driven (TMDB)
10. 🚀 **Start simple:** TMDB + Streaming Availability API for MVP, scale to JustWatch/Reelgood for advanced features

---

## Document Change Log

| Date | Author | Changes |
|------|--------|---------|
| 2025-12-05 | Research Agent | Initial comprehensive research and documentation |

---

## Sources

- [Watchmode - Streaming Availability Metadata API](https://api.watchmode.com/)
- [Streaming Availability API](https://www.movieofthenight.com/about/api/)
- [GitHub - Streaming Availability API](https://github.com/movieofthenight/ts-streaming-availability)
- [JustWatch Content Insights & Streaming Data API](https://media.justwatch.com/content-insights)
- [JustWatch API Documentation](https://apis.justwatch.com/docs/api/)
- [Reelgood for Business](https://data.reelgood.com/)
- [Reelgood Partner API](https://data.reelgood.com/products/reelgood-partner-api/)
- [Utelly - TV & OTT Content Discovery](https://www.utelly.com/)
- [Utelly API on RapidAPI](https://rapidapi.com/utelly/api/utelly)
- [TMDB Developer Portal](https://developer.themoviedb.org/)
- [TMDB Rate Limiting](https://developer.themoviedb.org/docs/rate-limiting)
- [OMDb API](https://www.omdbapi.com/)
- [ProxiesAPI - Does Netflix Allow Web Scraping](https://proxiesapi.com/articles/does-netflix-allow-web-scraping)
- [Ethical and Legal Considerations of Scraping Netflix Data](https://www.ottscrape.com/ethical-legal-considerations-netflix-data-scraping.php)
- [McCarthy Law Group - Is Web Scraping Legal? A 2025 Breakdown](https://mccarthylg.com/is-web-scraping-legal-a-2025-breakdown-of-what-you-need-to-know/)
- [ScrapingRocket - Web Scraping Legality & Ethics Guide](https://docs.scrapingrocket.com/blog/web-scraping-legality-ethics-guide)
- [Browserless - Is Web Scraping Legal in 2025?](https://www.browserless.io/blog/is-web-scraping-legal)
- [API7.ai - Real-Time Data with Streaming APIs](https://api7.ai/learning-center/api-101/real-time-data-with-streaming-apis)
- [Zuplo - What's the Best Movie Database API?](https://zuplo.com/learning-center/best-movie-api-imdb-vs-omdb-vs-tmdb)
- [Zuplo Blog - Guide to Real-Time Data Stream APIs](https://zuplo.com/blog/2025/04/04/guide-to-real-time-data-stream-apis)

---

**End of Document**
