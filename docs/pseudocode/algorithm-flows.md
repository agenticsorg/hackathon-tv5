# Media Gateway CLI - Algorithm Flows

## Visual Algorithm Representations

This document provides flowchart-style representations of key algorithms for the Media Gateway CLI.

---

## 1. Main CLI Execution Flow

```
┌─────────────────────────────────────────┐
│         CLI Application Start           │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│     Parse Command Line Arguments        │
│   (--options, flags, positional args)   │
└────────────────┬────────────────────────┘
                 │
                 ▼
          ┌──────┴──────┐
          │  Valid?     │
          └──┬───────┬──┘
             │       │
          No │       │ Yes
             │       │
             ▼       ▼
    ┌────────────┐  ┌─────────────────────┐
    │ Show Error │  │  Command Exists?    │
    │ Exit(2)    │  └──┬───────────────┬──┘
    └────────────┘     │               │
                    No │               │ Yes
                       │               │
                       ▼               ▼
              ┌────────────┐  ┌──────────────────┐
              │ Show Help  │  │ Auth Required?   │
              │ Exit(2)    │  └──┬────────────┬──┘
              └────────────┘     │            │
                              Yes│            │No
                                 │            │
                                 ▼            ▼
                        ┌──────────────┐  ┌──────────────┐
                        │Authenticated?│  │  Validate    │
                        └──┬────────┬──┘  │  Options     │
                           │        │     └──────┬───────┘
                        No │        │Yes         │
                           │        │            │
                           ▼        ▼            ▼
                    ┌────────────┐  └──────►┌─────────────┐
                    │Auth Error  │          │  Execute    │
                    │ Exit(3)    │          │  Command    │
                    └────────────┘          └──────┬──────┘
                                                   │
                                         ┌─────────┴─────────┐
                                         │   Success?        │
                                         └─────┬──────────┬──┘
                                               │          │
                                            Yes│          │No
                                               │          │
                                               ▼          ▼
                                      ┌────────────┐  ┌──────────┐
                                      │ Exit(0)    │  │ Exit(1)  │
                                      └────────────┘  └──────────┘
```

---

## 2. Search Command Flow

```
┌────────────────────────────────────────────┐
│      Search Command Execution              │
│   Input: query, filters, options           │
└──────────────────┬─────────────────────────┘
                   │
                   ▼
         ┌─────────────────────┐
         │ Normalize Query     │
         │ (trim, lowercase)   │
         └──────────┬──────────┘
                    │
                    ▼
          ┌─────────────────┐
          │ Query Length    │
          │    >= 2?        │
          └────┬────────┬───┘
               │        │
            No │        │ Yes
               │        │
               ▼        ▼
      ┌────────────┐  ┌──────────────────┐
      │ Return     │  │ Generate Cache   │
      │ Error      │  │ Key              │
      └────────────┘  └────────┬─────────┘
                               │
                               ▼
                      ┌────────────────┐
                      │ Check Cache    │
                      └────┬───────┬───┘
                           │       │
                     Found │       │ Not Found
                           │       │
                           ▼       ▼
                  ┌────────────┐  ┌─────────────────┐
                  │ Cached &   │  │ Build Filter    │
                  │ Not        │  │ Specification   │
                  │ Expired?   │  └────────┬────────┘
                  └────┬───┬───┘           │
                       │   │               │
                    Yes│   │No             │
                       │   │               │
                       │   └───────────────┤
                       │                   │
                       │                   ▼
                       │          ┌────────────────────┐
                       │          │ Execute API Search │
                       │          │ (with timeout)     │
                       │          └────────┬───────────┘
                       │                   │
                       │                   ▼
                       │          ┌────────────────────┐
                       │          │ Cache Results      │
                       │          │ (5 min TTL)        │
                       │          └────────┬───────────┘
                       │                   │
                       └───────────────────┤
                                           │
                                           ▼
                                  ┌────────────────┐
                                  │ Results Empty? │
                                  └────┬───────┬───┘
                                       │       │
                                    Yes│       │No
                                       │       │
                                       ▼       ▼
                              ┌────────────┐  ┌────────────────┐
                              │ Return     │  │ Interactive    │
                              │ Empty      │  │ Mode?          │
                              └────────────┘  └────┬───────┬───┘
                                                   │       │
                                                Yes│       │No
                                                   │       │
                                                   ▼       ▼
                                          ┌────────────┐  ┌──────────────┐
                                          │  Browse    │  │ Format &     │
                                          │  Results   │  │ Return       │
                                          └────────────┘  └──────────────┘
```

---

## 3. Interactive Browse Flow

```
┌─────────────────────────────────────┐
│     Interactive Browse Mode         │
│   Input: SearchResults, options     │
└──────────────┬──────────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Initialize State:    │
    │ - currentPage = 0    │
    │ - pageSize = 10      │
    └──────────┬───────────┘
               │
               ◄──────────────────┐
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Clear Screen         │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Display Header       │     │
    │ (total, query)       │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Get Current Page     │     │
    │ Items (10)           │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Display Table        │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Show Navigation      │     │
    │ Prompt               │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
    ┌──────────────────────┐     │
    │ Read User Input      │     │
    └──────────┬───────────┘     │
               │                  │
               ▼                  │
         ┌─────┴─────┐            │
         │  Command  │            │
         └─────┬─────┘            │
               │                  │
    ┌──────────┼────────────┬────┴──────┬─────────┬──────────┐
    │          │            │           │         │          │
    ▼          ▼            ▼           ▼         ▼          ▼
┌───────┐ ┌────────┐ ┌──────────┐ ┌────────┐ ┌──────┐ ┌────────┐
│"next" │ │"prev"  │ │ numeric  │ │"page n"│ │"add n"│ │"quit" │
└───┬───┘ └───┬────┘ └────┬─────┘ └───┬────┘ └───┬──┘ └───┬────┘
    │         │            │           │          │        │
    ▼         ▼            ▼           ▼          ▼        ▼
┌───────┐ ┌───────┐ ┌──────────┐ ┌────────┐ ┌─────────┐ ┌──────┐
│Has    │ │Current│ │Show Item │ │Jump to │ │Add to   │ │Return│
│Next?  │ │> 0?   │ │Details   │ │Page    │ │Watchlist│ │null  │
└───┬───┘ └───┬───┘ └────┬─────┘ └───┬────┘ └────┬────┘ └──────┘
    │Yes      │Yes        │           │           │
    ▼         ▼           │           ▼           │
┌───────┐ ┌───────┐      │      ┌────────┐       │
│page++ │ │page-- │      │      │Set page│       │
└───┬───┘ └───┬───┘      │      └───┬────┘       │
    │         │           │          │            │
    └─────────┴───────────┴──────────┴────────────┘
                          │
                          └──────────────────────┘
                                (Loop back to display)
```

---

## 4. Recommendation Scoring Algorithm

```
┌────────────────────────────────────────┐
│   Calculate Recommendation Score       │
│   Input: rec, profile, history         │
└──────────────┬─────────────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ score = rec.baseScore│
    │ (0-100 from engine)  │
    └──────────┬───────────┘
               │
               ▼
         ┌─────────────┐
         │ Has mood?   │
         └─────┬───┬───┘
               │   │
            Yes│   │No
               │   │
               ▼   │
    ┌───────────────────┐
    │ moodScore = calc  │
    │ score += mood*0.2 │
    └─────┬─────────────┘
          │
          ◄─────────────┘
          │
          ▼
    ┌─────────────┐
    │ Has context?│
    └─────┬───┬───┘
          │   │
       Yes│   │No
          │   │
          ▼   │
┌──────────────────────┐
│ contextScore = calc  │
│ score += ctx*0.15    │
└─────┬────────────────┘
      │
      ◄────────────────┘
      │
      ▼
┌─────────────┐
│ Has duration│
│ preference? │
└─────┬───┬───┘
      │   │
   Yes│   │No
      │   │
      ▼   │
┌──────────────────────┐
│ durationDiff = abs   │
│ score += dur*0.1     │
└─────┬────────────────┘
      │
      ◄────────────────┘
      │
      ▼
┌─────────────────────┐
│ Has platform prefs? │
└─────┬───┬───────────┘
      │   │
   Yes│   │No
      │   │
      ▼   │
┌──────────────────┐
│ Check matching   │
│ platforms        │
│ score += 5       │
└─────┬────────────┘
      │
      ◄──────────────┘
      │
      ▼
┌─────────────┐
│Has history? │
└─────┬───┬───┘
      │   │
   Yes│   │No
      │   │
      ▼   │
┌──────────────────┐
│ similarity = calc│
│ score += sim*0.25│
└─────┬────────────┘
      │
      ◄──────────────┘
      │
      ▼
┌─────────────────────┐
│ Freshness bonus     │
│ (if release < 2yrs) │
│ score += (2-yrs)*2  │
└─────┬───────────────┘
      │
      ▼
┌─────────────────────┐
│ Normalize: min(100, │
│ max(0, score))      │
└─────┬───────────────┘
      │
      ▼
┌─────────────────────┐
│ Return final score  │
└─────────────────────┘
```

---

## 5. Watchlist Sync Algorithm

```
┌────────────────────────────────────┐
│      Watchlist Sync Flow           │
└──────────────┬─────────────────────┘
               │
               ▼
    ┌──────────────────┐
    │ Get Local Items  │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │ Get Cloud Items  │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────────────────┐
    │ Calculate Diff:              │
    │ - toAdd (local → cloud)      │
    │ - toRemove (cloud → local)   │
    │ - toUpdate (conflicts)       │
    └──────────┬───────────────────┘
               │
               ▼
    ┌──────────────────┐
    │ For each toAdd   │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │ CloudAPI.add()   │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │For each toRemove │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │LocalAPI.remove() │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │For each toUpdate │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────────────┐
    │ Conflict Resolution:     │
    │ - Server wins for titles │
    │ - Client wins for notes  │
    │ - Latest timestamp wins  │
    └──────────┬───────────────┘
               │
               ▼
    ┌──────────────────┐
    │LocalAPI.update() │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────────┐
    │ Update lastSyncTime  │
    │ Save config          │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────┐
    │ Return summary   │
    └──────────────────┘
```

---

## 6. Deep Link Generation Flow

```
┌─────────────────────────────────────┐
│     Deep Link Generation            │
│  Input: media, platform, options    │
└──────────────┬──────────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Get Platform Base URL│
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Get Platform Media ID│
    └──────────┬───────────┘
               │
               ▼
         ┌─────┴──────┐
         │  Platform  │
         └─────┬──────┘
               │
    ┌──────────┼──────────┬──────────┬─────────┬────────┐
    │          │          │          │         │        │
    ▼          ▼          ▼          ▼         ▼        ▼
┌────────┐ ┌──────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Netflix │ │ Hulu │ │Disney+ │ │ Prime  │ │YouTube │ │Spotify │
└───┬────┘ └───┬──┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
    │          │        │          │          │          │
    ▼          ▼        ▼          ▼          ▼          ▼
┌────────────────────────────────────────────────────────────┐
│ Format platform-specific URL:                             │
│ - netflix: /watch/{id}?t={time}                           │
│ - hulu: /watch/{id}                                       │
│ - disney_plus: /video/{id}                                │
│ - prime: /gp/video/detail/{id}                            │
│ - youtube: /watch?v={id}&t={time}s                        │
│ - spotify: /track/{id}                                    │
└──────────────────────┬─────────────────────────────────────┘
                       │
                       ▼
            ┌──────────────────┐
            │ Add Quality Param│
            │ (if supported)   │
            └──────────┬───────┘
                       │
                       ▼
            ┌──────────────────┐
            │ Add Subtitles    │
            │ (if supported)   │
            └──────────┬───────┘
                       │
                       ▼
            ┌──────────────────────┐
            │ Return DeepLink:     │
            │ - url                │
            │ - platform           │
            │ - protocol           │
            │ - fallbackUrl        │
            └──────────────────────┘
```

---

## 7. Device Authorization Flow (OAuth)

```
┌────────────────────────────────┐
│    Device Authorization        │
└──────────────┬─────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Request Device Code  │
    │ from Auth Server     │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────────────────┐
    │ Receive:                     │
    │ - deviceCode                 │
    │ - userCode                   │
    │ - verificationUrl            │
    │ - expiresIn                  │
    └──────────┬───────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Display Instructions │
    │ to User              │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Show Spinner         │
    │ "Waiting..."         │
    └──────────┬───────────┘
               │
               ◄──────────────┐
               │              │
               ▼              │
    ┌──────────────────┐     │
    │ Wait 5 seconds   │     │
    └──────────┬───────┘     │
               │              │
               ▼              │
    ┌──────────────────────┐ │
    │ Poll Auth Server     │ │
    │ with deviceCode      │ │
    └──────────┬───────────┘ │
               │              │
               ▼              │
         ┌─────┴──────┐      │
         │   Status   │      │
         └─────┬──────┘      │
               │              │
    ┌──────────┼─────────┬───┴───────┬──────────┐
    │          │         │           │          │
    ▼          ▼         ▼           ▼          ▼
┌─────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌─────────┐
│Authori- │ │Pending │ │Denied  │ │Expired │ │Timeout  │
│zed      │ │        │ │        │ │        │ │(60 polls│
└───┬─────┘ └───┬────┘ └───┬────┘ └───┬────┘ └────┬────┘
    │           │          │          │           │
    │           │          │          │           │
    ▼           │          ▼          ▼           ▼
┌─────────────┐ │   ┌──────────┐ ┌──────────┐ ┌──────────┐
│ Save Tokens │ │   │ Return   │ │ Return   │ │ Return   │
│ Get User    │ │   │ Error    │ │ Error    │ │ Error    │
│ Save Config │ │   └──────────┘ └──────────┘ └──────────┘
│ Success!    │ │
└─────────────┘ │
                │
                └───(Loop: max 60 attempts = 5 min)
```

---

## 8. MCP Message Handling Flow

```
┌────────────────────────────────┐
│    MCP Message Received        │
│    (JSON-RPC 2.0)              │
└──────────────┬─────────────────┘
               │
               ▼
    ┌──────────────────┐
    │ Parse JSON       │
    └──────┬───────┬───┘
           │       │
        OK │       │ Parse Error
           │       │
           ▼       ▼
    ┌──────────┐  ┌────────────────┐
    │ Validate │  │ Send Error -32700│
    │ JSON-RPC │  │ "Parse error"   │
    │ Version  │  └────────────────┘
    └────┬─────┘
         │
         ▼
    ┌────────┐
    │ = 2.0? │
    └────┬───┴──┐
         │      │
      Yes│      │No
         │      │
         ▼      ▼
    ┌──────┐  ┌────────────────┐
    │ Get  │  │ Send Error -32600│
    │Method│  │ "Invalid Request"│
    └──┬───┘  └────────────────┘
       │
       ▼
    ┌────────────┐
    │ Method     │
    │ Exists?    │
    └────┬───┬───┘
         │   │
      Yes│   │No
         │   │
         ▼   ▼
    ┌──────┐ ┌────────────────┐
    │ Get  │ │ Send Error -32601│
    │Handler│ │ "Method not found"│
    └──┬───┘ └────────────────┘
       │
       ▼
    ┌────────────────┐
    │ Execute Handler│
    │ with params    │
    └──┬─────────────┘
       │
       ▼
    ┌──────┐
    │Success│
    └──┬───┴──┐
       │      │
    Yes│      │No (throws)
       │      │
       ▼      ▼
┌────────────┐ ┌────────────────┐
│ Build      │ │ Catch Error    │
│ Response:  │ │ Send Error -32603│
│ {          │ │ "Internal error"│
│  jsonrpc:  │ └────────────────┘
│  "2.0",    │
│  id: req.id│
│  result: {}│
│ }          │
└──────┬─────┘
       │
       ▼
┌────────────┐
│ Send JSON  │
│ Response   │
└────────────┘
```

---

## 9. Cache Strategy Flow

```
┌────────────────────────────────┐
│     Cache Lookup Strategy      │
└──────────────┬─────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Generate Cache Key   │
    │ (query + filters)    │
    └──────────┬───────────┘
               │
               ▼
    ┌──────────────────┐
    │ Hash(key) → id   │
    └──────────┬───────┘
               │
               ▼
    ┌──────────────────┐
    │ Get from cache   │
    └──────┬───────┬───┘
           │       │
       Found│      │Not Found
           │       │
           ▼       ▼
    ┌──────────┐  ┌────────────┐
    │ Check    │  │ Return     │
    │ TTL      │  │ MISS       │
    └────┬─────┘  └────────────┘
         │
         ▼
    ┌────────┐
    │ Expired│
    └────┬───┴──┐
         │      │
      Yes│      │No
         │      │
         ▼      ▼
    ┌────────┐ ┌────────────┐
    │ Delete │ │ Return HIT │
    │ Return │ │ with data  │
    │ MISS   │ └────────────┘
    └────────┘

┌────────────────────────────────┐
│      Cache Write Strategy      │
└──────────────┬─────────────────┘
               │
               ▼
    ┌──────────────────┐
    │ Check cache size │
    └──────┬───────┬───┘
           │       │
     Full  │       │ Has Space
           │       │
           ▼       │
    ┌──────────┐  │
    │ Evict LRU│  │
    │ Entry    │  │
    └────┬─────┘  │
         │        │
         └────────┤
                  │
                  ▼
         ┌────────────────┐
         │ Set value with │
         │ TTL timestamp  │
         └────────────────┘
```

---

## 10. Error Recovery Flow

```
┌────────────────────────────────┐
│    Error Occurs in Command     │
└──────────────┬─────────────────┘
               │
               ▼
         ┌─────┴──────┐
         │ Error Type │
         └─────┬──────┘
               │
    ┌──────────┼──────────┬─────────┬──────────┐
    │          │          │         │          │
    ▼          ▼          ▼         ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐ ┌─────────┐
│Network │ │ Auth   │ │Timeout │ │Parse │ │ Other   │
└───┬────┘ └───┬────┘ └───┬────┘ └──┬───┘ └────┬────┘
    │          │          │         │          │
    ▼          ▼          ▼         ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐ ┌─────────┐
│ Retry  │ │ Clear  │ │ Partial│ │ Show │ │ Log     │
│ with   │ │ Tokens │ │ Results│ │ Help │ │ Stack   │
│Backoff │ │ Prompt │ │ if any │ └──┬───┘ │ if debug│
└───┬────┘ │ Login  │ └───┬────┘    │     └────┬────┘
    │      └────────┘     │         │          │
    ▼                     │         │          │
┌────────┐               │         │          │
│Success?│               │         │          │
└───┬──┬─┘               │         │          │
    │  │                 │         │          │
 Yes│  │No               │         │          │
    │  │                 │         │          │
    ▼  └─────────────────┴─────────┴──────────┤
┌────────┐                                    │
│ Return │                                    │
│ Result │                                    │
└────────┘                                    │
                                              ▼
                                    ┌──────────────┐
                                    │ Return Error │
                                    │ with Exit    │
                                    │ Code         │
                                    └──────────────┘
```

---

## Algorithm Complexity Reference

### Quick Lookup Table

| Algorithm | Time | Space | Notes |
|-----------|------|-------|-------|
| CLI Parse | O(n) | O(n) | n = args |
| Search (cached) | O(1) | O(r) | r = results |
| Search (API) | O(n log n) | O(r) | Sorting |
| Recommend Score | O(1) | O(1) | Per item |
| Recommend Sort | O(n log n) | O(n) | n = items |
| Watchlist Sync | O(w) | O(w) | w = size |
| Device List | O(d log d) | O(d) | d = devices |
| Cast Platform | O(p × c) | O(1) | p×c small |
| Auth Poll | O(t) | O(1) | t = attempts |
| MCP Handle | O(1) | O(1) | Per message |
| Table Format | O(r × c) | O(r × c) | Grid |
| Cache Lookup | O(1) | O(k) | k = cache size |

---

## Design Decision Summary

### Key Optimizations

1. **Caching**: 5-minute TTL reduces API calls by ~80%
2. **Pagination**: Constant memory usage for large result sets
3. **Lazy Loading**: Commands loaded on-demand
4. **Connection Pooling**: Reuse HTTP connections
5. **Parallel Requests**: Independent operations run concurrently

### Trade-offs

1. **Cache vs Freshness**: 5-min cache balances speed vs data freshness
2. **Memory vs Speed**: LRU cache bounded at 1000 entries
3. **Complexity vs Features**: Interactive mode adds UX at cost of code complexity
4. **Security vs Convenience**: Token storage encrypted but requires OS keychain

---

## Implementation Priority

1. **Phase 1**: Core framework, parsing, basic commands
2. **Phase 2**: API integration, auth, caching
3. **Phase 3**: Interactive modes, advanced features
4. **Phase 4**: MCP server, optimizations
5. **Phase 5**: Polish, error handling, monitoring

---

This visual representation complements the detailed pseudocode and provides a clear roadmap for implementation.
