# Media Gateway CLI - SPARC Pseudocode Phase

**Document Version**: 1.0.0
**Phase**: Pseudocode
**Last Updated**: 2025-12-06

## 1. COMMAND PARSER FRAMEWORK

### 1.1 Core CLI Application Structure

```
CLASS CLIApp:
    PROPERTIES:
        commands: Map<string, Command>
        config: AppConfig
        logger: Logger
        version: string

    CONSTANTS:
        EXIT_SUCCESS = 0
        EXIT_ERROR = 1
        EXIT_INVALID_ARGS = 2
        EXIT_AUTH_REQUIRED = 3
        EXIT_NETWORK_ERROR = 4

    FUNCTION initialize():
        INPUT: None
        OUTPUT: void

        BEGIN
            // Load configuration
            config ← ConfigLoader.load()

            // Initialize logger with config
            logger ← Logger.create(config.logLevel, config.colorMode)

            // Register all commands
            RegisterCommands()

            // Setup signal handlers
            RegisterSignalHandlers()
        END

    FUNCTION run(args: string[]):
        INPUT: args - Command line arguments
        OUTPUT: exitCode (integer)

        BEGIN
            TRY
                // Parse arguments
                parsed ← ParseArguments(args)

                // Validate command exists
                IF NOT commands.has(parsed.command) THEN
                    logger.error("Unknown command: " + parsed.command)
                    ShowHelp()
                    RETURN EXIT_INVALID_ARGS
                END IF

                // Get command instance
                command ← commands.get(parsed.command)

                // Validate required options
                validation ← ValidateOptions(command, parsed.options)
                IF NOT validation.isValid THEN
                    logger.error(validation.errors)
                    RETURN EXIT_INVALID_ARGS
                END IF

                // Check authentication requirement
                IF command.requiresAuth AND NOT IsAuthenticated() THEN
                    logger.error("Authentication required. Run: cli auth login")
                    RETURN EXIT_AUTH_REQUIRED
                END IF

                // Determine execution mode
                IF IsInteractiveMode(parsed.options) THEN
                    result ← command.runInteractive(parsed.options)
                ELSE
                    result ← command.runBatch(parsed.options)
                END IF

                // Output result
                OutputResult(result, parsed.options.format)

                RETURN result.success ? EXIT_SUCCESS : EXIT_ERROR

            CATCH NetworkError AS e
                logger.error("Network error: " + e.message)
                RETURN EXIT_NETWORK_ERROR

            CATCH AuthenticationError AS e
                logger.error("Authentication error: " + e.message)
                RETURN EXIT_AUTH_REQUIRED

            CATCH error AS e
                logger.error("Unexpected error: " + e.message)
                IF config.debug THEN
                    logger.debug(e.stackTrace)
                END IF
                RETURN EXIT_ERROR
            END TRY
        END

    FUNCTION RegisterCommands():
        BEGIN
            commands.set("search", NEW SearchCommand())
            commands.set("recommend", NEW RecommendCommand())
            commands.set("watchlist", NEW WatchlistCommand())
            commands.set("devices", NEW DevicesCommand())
            commands.set("cast", NEW CastCommand())
            commands.set("auth", NEW AuthCommand())
            commands.set("mcp", NEW MCPCommand())
            commands.set("config", NEW ConfigCommand())
        END
```

**Time Complexity**: O(1) for command lookup
**Space Complexity**: O(n) where n = number of registered commands

---

## 2. ARGUMENT PARSING ALGORITHM

```
ALGORITHM: ParseArguments
INPUT: args (array of strings)
OUTPUT: ParsedCommand object

DATA STRUCTURES:
    ParsedCommand:
        command: string
        options: Map<string, any>
        flags: Set<string>
        positionalArgs: string[]

BEGIN
    result ← NEW ParsedCommand()
    currentOption ← null

    FOR i = 0 TO args.length - 1 DO
        arg ← args[i]

        // Long option: --option or --option=value
        IF arg.startsWith("--") THEN
            IF arg.contains("=") THEN
                [key, value] ← arg.split("=", 2)
                result.options.set(key.substring(2), ParseValue(value))
                currentOption ← null
            ELSE
                key ← arg.substring(2)
                currentOption ← key
                // Next arg might be value
            END IF

        // Short option: -o or -abc (combined)
        ELSE IF arg.startsWith("-") AND arg.length > 1 THEN
            IF arg.length = 2 THEN
                // Single short option
                flag ← arg.substring(1)
                currentOption ← ExpandShortOption(flag)
            ELSE
                // Combined short options: -abc = -a -b -c
                FOR j = 1 TO arg.length - 1 DO
                    flag ← arg[j]
                    expandedFlag ← ExpandShortOption(flag)
                    result.flags.add(expandedFlag)
                END FOR
                currentOption ← null
            END IF

        // Value for previous option
        ELSE IF currentOption IS NOT null THEN
            result.options.set(currentOption, ParseValue(arg))
            currentOption ← null

        // Command or positional argument
        ELSE
            IF result.command IS null THEN
                result.command ← arg
            ELSE
                result.positionalArgs.append(arg)
            END IF
        END IF
    END FOR

    // Handle flag-only options
    IF currentOption IS NOT null THEN
        result.flags.add(currentOption)
    END IF

    RETURN result
END

SUBROUTINE: ParseValue
INPUT: value (string)
OUTPUT: parsed value (string, number, boolean, or array)

BEGIN
    // Boolean values
    IF value IN ["true", "yes", "1"] THEN
        RETURN true
    ELSE IF value IN ["false", "no", "0"] THEN
        RETURN false
    END IF

    // Number values
    IF IsNumeric(value) THEN
        RETURN ParseNumber(value)
    END IF

    // Array values (comma-separated)
    IF value.contains(",") THEN
        RETURN value.split(",").map(trim)
    END IF

    // String value
    RETURN value
END
```

**Time Complexity**: O(n) where n = number of arguments
**Space Complexity**: O(n) for storing parsed result

---

## 3. SEARCH COMMAND

```
ALGORITHM: SearchCommand.execute
INPUT: query (string), options (SearchOptions)
OUTPUT: SearchResult

DATA STRUCTURES:
    SearchOptions:
        type: MediaType[]           // ["movie", "tv", "music"]
        genre: string[]
        platform: string[]
        year: Range<integer>
        rating: Range<float>
        limit: integer              // Default: 20
        offset: integer             // Default: 0
        sortBy: SortField
        sortOrder: "asc" | "desc"
        interactive: boolean
        format: OutputFormat        // "table" | "json" | "pretty"

    SearchResult:
        items: MediaItem[]
        total: integer
        hasMore: boolean
        metadata: ResultMetadata

CONSTANTS:
    MAX_RESULTS_PER_PAGE = 100
    DEFAULT_RESULTS = 20
    CACHE_TTL_SECONDS = 300

BEGIN
    // Step 1: Validate and normalize query
    normalizedQuery ← NormalizeQuery(query)
    IF normalizedQuery.length < 2 THEN
        RETURN Error("Query too short. Minimum 2 characters.")
    END IF

    // Step 2: Check cache
    cacheKey ← GenerateCacheKey(normalizedQuery, options)
    cached ← SearchCache.get(cacheKey)

    IF cached IS NOT null AND NOT cached.isExpired() THEN
        logger.debug("Cache hit for query: " + query)

        IF options.interactive THEN
            RETURN BrowseResults(cached.results, options)
        ELSE
            RETURN FormatResults(cached.results, options)
        END IF
    END IF

    // Step 3: Build search filters
    filters ← BuildFilters(options)

    // Step 4: Execute search with timeout
    startTime ← GetCurrentTime()

    WITH TIMEOUT(30000) DO
        results ← MediaGatewayAPI.search({
            query: normalizedQuery,
            filters: filters,
            limit: options.limit,
            offset: options.offset,
            sortBy: options.sortBy,
            sortOrder: options.sortOrder
        })
    END TIMEOUT

    searchTime ← GetCurrentTime() - startTime

    // Step 5: Cache results
    SearchCache.set(cacheKey, results, CACHE_TTL_SECONDS)

    // Step 6: Process results
    IF results.items.length = 0 THEN
        RETURN EmptyResult("No results found for: " + query)
    END IF

    // Step 7: Interactive or batch output
    IF options.interactive THEN
        RETURN BrowseResults(results, options)
    ELSE
        RETURN FormatResults(results, options)
    END IF
END

SUBROUTINE: BuildFilters
INPUT: options (SearchOptions)
OUTPUT: FilterSpec

BEGIN
    filters ← NEW FilterSpec()

    // Type filter
    IF options.type IS NOT empty THEN
        filters.add({
            field: "type",
            operator: "IN",
            value: options.type
        })
    END IF

    // Genre filter (OR condition)
    IF options.genre IS NOT empty THEN
        filters.add({
            field: "genre",
            operator: "IN",
            value: options.genre
        })
    END IF

    // Platform filter (AND condition)
    IF options.platform IS NOT empty THEN
        FOR EACH platform IN options.platform DO
            filters.add({
                field: "availableOn",
                operator: "CONTAINS",
                value: platform
            })
        END FOR
    END IF

    // Year range filter
    IF options.year IS NOT null THEN
        IF options.year.min IS NOT null THEN
            filters.add({
                field: "releaseYear",
                operator: ">=",
                value: options.year.min
            })
        END IF
        IF options.year.max IS NOT null THEN
            filters.add({
                field: "releaseYear",
                operator: "<=",
                value: options.year.max
            })
        END IF
    END IF

    // Rating filter
    IF options.rating IS NOT null THEN
        filters.add({
            field: "rating",
            operator: ">=",
            value: options.rating.min
        })
    END IF

    RETURN filters
END

SUBROUTINE: BrowseResults (Interactive Mode)
INPUT: results (SearchResult), options (SearchOptions)
OUTPUT: InteractiveResult

BEGIN
    currentPage ← 0
    pageSize ← 10
    selection ← null

    WHILE true DO
        // Display current page
        ClearScreen()
        DisplaySearchHeader(results.total, results.query)

        pageItems ← GetPage(results.items, currentPage, pageSize)
        DisplayResultsTable(pageItems, currentPage * pageSize)

        // Display navigation prompt
        prompt ← BuildNavigationPrompt(currentPage, pageSize, results.total)
        userInput ← ReadUserInput(prompt)

        CASE userInput OF
            "n", "next":
                IF HasNextPage(currentPage, pageSize, results.total) THEN
                    currentPage ← currentPage + 1
                ELSE
                    ShowMessage("Last page reached")
                END IF

            "p", "prev", "previous":
                IF currentPage > 0 THEN
                    currentPage ← currentPage - 1
                ELSE
                    ShowMessage("First page")
                END IF

            "q", "quit", "exit":
                RETURN null

            // Numeric selection
            WHEN IsNumeric(userInput):
                index ← ParseInteger(userInput)
                IF index >= 0 AND index < pageItems.length THEN
                    selection ← pageItems[index]
                    RETURN ShowMediaDetails(selection)
                ELSE
                    ShowMessage("Invalid selection")
                END IF

            // Jump to page
            WHEN userInput.startsWith("page "):
                pageNum ← ParseInteger(userInput.substring(5))
                IF IsValidPage(pageNum, pageSize, results.total) THEN
                    currentPage ← pageNum - 1
                ELSE
                    ShowMessage("Invalid page number")
                END IF

            // Add to watchlist
            WHEN userInput.startsWith("add "):
                index ← ParseInteger(userInput.substring(4))
                IF index >= 0 AND index < pageItems.length THEN
                    AddToWatchlist(pageItems[index])
                    ShowMessage("Added to watchlist")
                END IF

            OTHERWISE:
                ShowMessage("Invalid command. Type 'help' for options.")
        END CASE
    END WHILE
END
```

**Time Complexity**:
- Cache lookup: O(1)
- Filter building: O(f) where f = number of filters
- API search: O(n log n) where n = result set size (server-side)
- Interactive browsing: O(1) per page display

**Space Complexity**: O(r) where r = number of results cached

---

## 4. RECOMMEND COMMAND

```
ALGORITHM: RecommendCommand.execute
INPUT: context (string), options (RecommendOptions)
OUTPUT: RecommendationResult

DATA STRUCTURES:
    RecommendOptions:
        mood: string                // "action", "relaxing", "intellectual"
        context: string             // "date night", "family time", "solo"
        duration: integer           // Max duration in minutes
        previousWatched: string[]   // IDs of previously watched content
        excludeGenres: string[]
        platform: string[]
        includeRationale: boolean   // Show explanation
        limit: integer

    RecommendationItem:
        media: MediaItem
        score: float
        rationale: string
        matchFactors: MatchFactor[]

BEGIN
    // Step 1: Validate input
    IF context IS empty AND options.mood IS empty THEN
        RETURN Error("Provide either context or mood")
    END IF

    // Step 2: Build recommendation profile
    profile ← BuildUserProfile({
        context: context,
        mood: options.mood,
        duration: options.duration,
        excludeGenres: options.excludeGenres,
        platform: options.platform
    })

    // Step 3: Get user history for personalization
    userHistory ← null
    IF IsAuthenticated() THEN
        userHistory ← WatchlistAPI.getHistory()

        // Add watch history to profile
        IF options.previousWatched IS NOT empty THEN
            profile.watchedIds ← profile.watchedIds.concat(options.previousWatched)
        END IF
    END IF

    // Step 4: Call recommendation engine
    WITH TIMEOUT(45000) DO
        recommendations ← MediaGatewayAPI.recommend({
            profile: profile,
            history: userHistory,
            limit: options.limit,
            includeExplanations: options.includeRationale
        })
    END TIMEOUT

    // Step 5: Score and rank recommendations
    scoredRecs ← []
    FOR EACH rec IN recommendations DO
        score ← CalculateRecommendationScore(rec, profile, userHistory)

        scoredRecs.append({
            media: rec.media,
            score: score,
            rationale: rec.explanation,
            matchFactors: rec.factors
        })
    END FOR

    // Sort by score descending
    scoredRecs.sortBy(item => item.score, "desc")

    // Step 6: Format output
    RETURN FormatRecommendations(scoredRecs, options)
END

SUBROUTINE: CalculateRecommendationScore
INPUT: rec (Recommendation), profile (UserProfile), history (WatchHistory)
OUTPUT: score (float)

ALGORITHM: Weighted scoring with personalization

BEGIN
    score ← 0.0

    // Base score from recommendation engine (0-100)
    score ← rec.baseScore

    // Mood match boost (up to +20)
    IF profile.mood IS NOT null THEN
        moodScore ← CalculateMoodMatch(rec.media, profile.mood)
        score ← score + (moodScore * 0.2)
    END IF

    // Context appropriateness (up to +15)
    IF profile.context IS NOT null THEN
        contextScore ← CalculateContextMatch(rec.media, profile.context)
        score ← score + (contextScore * 0.15)
    END IF

    // Duration preference (up to +10)
    IF profile.duration IS NOT null THEN
        durationDiff ← ABS(rec.media.duration - profile.duration)
        durationScore ← MAX(0, 100 - (durationDiff / 5))
        score ← score + (durationScore * 0.1)
    END IF

    // Platform availability (+5 if available on user's platforms)
    IF profile.platform IS NOT empty THEN
        hasMatchingPlatform ← false
        FOR EACH platform IN profile.platform DO
            IF rec.media.platforms.contains(platform) THEN
                hasMatchingPlatform ← true
                BREAK
            END IF
        END FOR
        IF hasMatchingPlatform THEN
            score ← score + 5
        END IF
    END IF

    // Personalization based on history (up to +25)
    IF history IS NOT null THEN
        similarityScore ← CalculateHistorySimilarity(rec.media, history)
        score ← score + (similarityScore * 0.25)
    END IF

    // Freshness bonus (newer content gets slight boost)
    yearsSinceRelease ← CurrentYear() - rec.media.releaseYear
    IF yearsSinceRelease < 2 THEN
        score ← score + (2 - yearsSinceRelease) * 2
    END IF

    // Normalize to 0-100
    RETURN MIN(100, MAX(0, score))
END
```

**Time Complexity**: O(n log n) where n = number of recommendations (sorting)
**Space Complexity**: O(n) for storing scored recommendations

---

## 5. WATCHLIST COMMAND

```
ALGORITHM: WatchlistCommand.execute
INPUT: action (string), options (WatchlistOptions)
OUTPUT: WatchlistResult

DATA STRUCTURES:
    WatchlistActions:
        LIST = "list"
        ADD = "add"
        REMOVE = "remove"
        SYNC = "sync"
        CLEAR = "clear"

    WatchlistOptions:
        mediaId: string
        page: integer
        pageSize: integer
        sortBy: "added" | "title" | "rating"
        filter: WatchlistFilter
        confirm: boolean

BEGIN
    // Authentication check
    IF NOT IsAuthenticated() THEN
        RETURN Error("Watchlist requires authentication. Run: cli auth login")
    END IF

    CASE action OF
        WatchlistActions.LIST:
            RETURN ListWatchlist(options)

        WatchlistActions.ADD:
            IF options.mediaId IS empty THEN
                RETURN Error("Media ID required for add action")
            END IF
            RETURN AddToWatchlist(options.mediaId, options.confirm)

        WatchlistActions.REMOVE:
            IF options.mediaId IS empty THEN
                RETURN Error("Media ID required for remove action")
            END IF
            RETURN RemoveFromWatchlist(options.mediaId, options.confirm)

        WatchlistActions.SYNC:
            RETURN SyncWatchlist(options.confirm)

        WatchlistActions.CLEAR:
            RETURN ClearWatchlist(options.confirm)

        OTHERWISE:
            RETURN Error("Invalid action: " + action)
    END CASE
END

SUBROUTINE: ListWatchlist
INPUT: options (WatchlistOptions)
OUTPUT: WatchlistResult

BEGIN
    // Fetch watchlist with pagination
    result ← WatchlistAPI.getWatchlist({
        page: options.page || 1,
        pageSize: options.pageSize || 20,
        sortBy: options.sortBy || "added",
        filter: options.filter
    })

    IF result.items.length = 0 THEN
        RETURN EmptyResult("Your watchlist is empty")
    END IF

    // Display with pagination
    totalPages ← CEIL(result.total / options.pageSize)

    logger.info("Watchlist (" + result.total + " items)")
    logger.info("Page " + options.page + " of " + totalPages)
    logger.divider()

    // Format as table
    table ← FormatTable(result.items, {
        columns: ["#", "Title", "Type", "Year", "Rating", "Added"],
        columnWidths: [4, 40, 10, 6, 8, 12]
    })

    logger.info(table)
    logger.divider()

    // Show navigation hints
    IF options.page < totalPages THEN
        logger.info("Next page: cli watchlist list --page " + (options.page + 1))
    END IF

    RETURN {
        success: true,
        items: result.items,
        total: result.total,
        page: options.page,
        totalPages: totalPages
    }
END

SUBROUTINE: AddToWatchlist
INPUT: mediaId (string), confirm (boolean)
OUTPUT: Result

BEGIN
    // Fetch media details
    media ← MediaGatewayAPI.getMedia(mediaId)

    IF media IS null THEN
        RETURN Error("Media not found: " + mediaId)
    END IF

    // Check if already in watchlist
    exists ← WatchlistAPI.contains(mediaId)
    IF exists THEN
        RETURN Warning("Already in watchlist: " + media.title)
    END IF

    // Confirmation prompt if needed
    IF confirm THEN
        message ← "Add to watchlist?\n" +
                  "Title: " + media.title + "\n" +
                  "Type: " + media.type + "\n" +
                  "Year: " + media.year

        confirmed ← PromptConfirmation(message)
        IF NOT confirmed THEN
            RETURN Cancelled("Add cancelled")
        END IF
    END IF

    // Add to watchlist
    result ← WatchlistAPI.add(mediaId)

    IF result.success THEN
        logger.success("Added to watchlist: " + media.title)

        // Trigger sync if enabled
        IF config.autoSync THEN
            SyncWatchlist(false)
        END IF

        RETURN {
            success: true,
            mediaId: mediaId,
            title: media.title
        }
    ELSE
        RETURN Error("Failed to add: " + result.error)
    END IF
END

SUBROUTINE: SyncWatchlist
INPUT: confirm (boolean)
OUTPUT: SyncResult

BEGIN
    IF confirm THEN
        confirmed ← PromptConfirmation("Force sync watchlist with cloud?")
        IF NOT confirmed THEN
            RETURN Cancelled("Sync cancelled")
        END IF
    END IF

    logger.info("Syncing watchlist...")
    spinner ← ShowSpinner("Syncing")

    TRY
        // Get local watchlist
        localItems ← WatchlistAPI.getLocalItems()

        // Get cloud watchlist
        cloudItems ← WatchlistAPI.getCloudItems()

        // Calculate diff
        diff ← CalculateWatchlistDiff(localItems, cloudItems)

        logger.info("Found " + diff.toAdd.length + " items to upload")
        logger.info("Found " + diff.toRemove.length + " items to remove")
        logger.info("Found " + diff.toUpdate.length + " items to update")

        // Apply changes
        FOR EACH item IN diff.toAdd DO
            WatchlistAPI.cloudAdd(item)
        END FOR

        FOR EACH item IN diff.toRemove DO
            WatchlistAPI.localRemove(item)
        END FOR

        FOR EACH item IN diff.toUpdate DO
            WatchlistAPI.localUpdate(item)
        END FOR

        // Update last sync timestamp
        config.lastWatchlistSync ← GetCurrentTime()
        ConfigManager.save(config)

        spinner.stop()
        logger.success("Watchlist synced successfully")

        RETURN {
            success: true,
            added: diff.toAdd.length,
            removed: diff.toRemove.length,
            updated: diff.toUpdate.length
        }

    CATCH error AS e
        spinner.stop()
        logger.error("Sync failed: " + e.message)
        RETURN Error(e.message)
    END TRY
END
```

**Time Complexity**:
- List: O(1) with pagination
- Add: O(1)
- Remove: O(1)
- Sync: O(n) where n = watchlist size

**Space Complexity**: O(n) for storing watchlist items

---

## 6. DEVICES COMMAND

```
ALGORITHM: DevicesCommand.execute
INPUT: action (string), options (DeviceOptions)
OUTPUT: DeviceResult

DATA STRUCTURES:
    DeviceActions:
        LIST = "list"
        RENAME = "rename"
        REMOVE = "remove"
        REGISTER = "register"

    Device:
        id: string
        name: string
        type: DeviceType          // "tv", "mobile", "browser", "streaming_device"
        platform: string          // "roku", "fire_tv", "chromecast", etc.
        lastSeen: timestamp
        ipAddress: string
        capabilities: string[]

BEGIN
    CASE action OF
        DeviceActions.LIST:
            RETURN ListDevices(options)

        DeviceActions.RENAME:
            RETURN RenameDevice(options)

        DeviceActions.REMOVE:
            RETURN RemoveDevice(options)

        DeviceActions.REGISTER:
            RETURN RegisterDevice(options)

        OTHERWISE:
            RETURN Error("Invalid action: " + action)
    END CASE
END

SUBROUTINE: ListDevices
INPUT: options (DeviceOptions)
OUTPUT: DeviceListResult

BEGIN
    // Fetch registered devices
    devices ← DeviceAPI.getDevices()

    IF devices.length = 0 THEN
        logger.info("No devices registered")
        logger.info("To register a device: cli devices register")
        RETURN EmptyResult()
    END IF

    // Sort by last seen (most recent first)
    devices.sortBy(device => device.lastSeen, "desc")

    // Format as table
    table ← FormatTable(devices, {
        columns: ["ID", "Name", "Type", "Platform", "Last Seen", "Status"],
        columnWidths: [20, 25, 15, 15, 20, 10],
        formatters: {
            lastSeen: (timestamp) => FormatRelativeTime(timestamp),
            status: (device) => device.isOnline ? "Online" : "Offline"
        }
    })

    logger.info("Registered Devices (" + devices.length + ")")
    logger.divider()
    logger.info(table)
    logger.divider()

    RETURN {
        success: true,
        devices: devices
    }
END

SUBROUTINE: RenameDevice
INPUT: options (DeviceOptions)
OUTPUT: Result

BEGIN
    deviceId ← options.deviceId
    newName ← options.name

    // Validate inputs
    IF deviceId IS empty THEN
        // Interactive selection
        devices ← DeviceAPI.getDevices()
        deviceId ← PromptDeviceSelection(devices)
    END IF

    IF newName IS empty THEN
        // Interactive prompt
        currentDevice ← DeviceAPI.getDevice(deviceId)
        newName ← PromptInput("Enter new name for '" + currentDevice.name + "': ")
    END IF

    // Validate name
    IF newName.length < 2 OR newName.length > 50 THEN
        RETURN Error("Device name must be 2-50 characters")
    END IF

    // Update device
    result ← DeviceAPI.updateDevice(deviceId, { name: newName })

    IF result.success THEN
        logger.success("Device renamed to: " + newName)
        RETURN result
    ELSE
        RETURN Error("Failed to rename device: " + result.error)
    END IF
END

SUBROUTINE: RemoveDevice
INPUT: options (DeviceOptions)
OUTPUT: Result

BEGIN
    deviceId ← options.deviceId

    IF deviceId IS empty THEN
        // Interactive selection
        devices ← DeviceAPI.getDevices()
        deviceId ← PromptDeviceSelection(devices)
    END IF

    // Get device details
    device ← DeviceAPI.getDevice(deviceId)
    IF device IS null THEN
        RETURN Error("Device not found: " + deviceId)
    END IF

    // Confirmation
    confirmed ← PromptConfirmation(
        "Remove device '" + device.name + "'?\n" +
        "This cannot be undone."
    )

    IF NOT confirmed THEN
        RETURN Cancelled("Remove cancelled")
    END IF

    // Remove device
    result ← DeviceAPI.removeDevice(deviceId)

    IF result.success THEN
        logger.success("Device removed: " + device.name)
        RETURN result
    ELSE
        RETURN Error("Failed to remove device: " + result.error)
    END IF
END
```

**Time Complexity**: O(n) for listing and sorting devices
**Space Complexity**: O(n) for storing device list

---

## 7. CAST COMMAND

```
ALGORITHM: CastCommand.execute
INPUT: mediaId (string), options (CastOptions)
OUTPUT: CastResult

DATA STRUCTURES:
    CastOptions:
        deviceId: string
        platform: string
        startTime: integer        // Seconds
        quality: "auto" | "sd" | "hd" | "4k"
        subtitles: boolean
        autoplay: boolean

    DeepLink:
        url: string
        platform: string
        protocol: string          // "intent://", "https://", etc.
        fallbackUrl: string

BEGIN
    // Step 1: Validate media
    media ← MediaGatewayAPI.getMedia(mediaId)
    IF media IS null THEN
        RETURN Error("Media not found: " + mediaId)
    END IF

    // Step 2: Device selection
    deviceId ← options.deviceId

    IF deviceId IS empty THEN
        // Interactive device selection
        devices ← DeviceAPI.getDevices()
        onlineDevices ← devices.filter(d => d.isOnline)

        IF onlineDevices.length = 0 THEN
            RETURN Error("No online devices found")
        END IF

        deviceId ← PromptDeviceSelection(onlineDevices)
    END IF

    device ← DeviceAPI.getDevice(deviceId)
    IF device IS null THEN
        RETURN Error("Device not found: " + deviceId)
    END IF

    // Step 3: Platform selection
    platform ← options.platform

    IF platform IS empty THEN
        // Find compatible platforms
        compatiblePlatforms ← FindCompatiblePlatforms(media, device)

        IF compatiblePlatforms.length = 0 THEN
            RETURN Error("No compatible platforms found for this media on " + device.name)
        END IF

        IF compatiblePlatforms.length = 1 THEN
            platform ← compatiblePlatforms[0]
        ELSE
            platform ← PromptPlatformSelection(compatiblePlatforms)
        END IF
    END IF

    // Step 4: Generate deep link
    deepLink ← GenerateDeepLink(media, platform, {
        startTime: options.startTime || 0,
        quality: options.quality,
        subtitles: options.subtitles,
        autoplay: options.autoplay
    })

    // Step 5: Cast to device
    logger.info("Casting '" + media.title + "' to " + device.name + "...")

    result ← DeviceAPI.cast(deviceId, deepLink)

    IF result.success THEN
        logger.success("Successfully cast to " + device.name)
        logger.info("Platform: " + platform)
        logger.info("Deep link: " + deepLink.url)

        RETURN {
            success: true,
            device: device,
            platform: platform,
            deepLink: deepLink
        }
    ELSE
        RETURN Error("Cast failed: " + result.error)
    END IF
END

SUBROUTINE: GenerateDeepLink
INPUT: media (MediaItem), platform (string), options (DeepLinkOptions)
OUTPUT: DeepLink

ALGORITHM: Platform-specific deep link generation

BEGIN
    baseUrl ← GetPlatformBaseUrl(platform)
    mediaId ← GetPlatformMediaId(media, platform)

    CASE platform OF
        "netflix":
            url ← "https://www.netflix.com/watch/" + mediaId
            IF options.startTime > 0 THEN
                url ← url + "?t=" + options.startTime
            END IF
            protocol ← "nflx://"

        "hulu":
            url ← "https://www.hulu.com/watch/" + mediaId
            protocol ← "hulu://"

        "disney_plus":
            url ← "https://www.disneyplus.com/video/" + mediaId
            protocol ← "disneyplus://"

        "amazon_prime":
            url ← "https://www.amazon.com/gp/video/detail/" + mediaId
            protocol ← "aiv://"

        "youtube":
            url ← "https://www.youtube.com/watch?v=" + mediaId
            IF options.startTime > 0 THEN
                url ← url + "&t=" + options.startTime + "s"
            END IF
            protocol ← "vnd.youtube://"

        "spotify":
            url ← "https://open.spotify.com/track/" + mediaId
            protocol ← "spotify://"

        OTHERWISE:
            // Generic web URL
            url ← baseUrl + "/" + mediaId
            protocol ← "https://"
    END CASE

    // Add quality parameter if supported
    IF options.quality AND platform IN ["netflix", "amazon_prime"] THEN
        url ← AddQueryParam(url, "quality", options.quality)
    END IF

    // Add subtitles parameter if supported
    IF options.subtitles AND platform IN ["netflix", "hulu", "disney_plus"] THEN
        url ← AddQueryParam(url, "subtitles", "true")
    END IF

    RETURN {
        url: url,
        platform: platform,
        protocol: protocol,
        fallbackUrl: url
    }
END

SUBROUTINE: FindCompatiblePlatforms
INPUT: media (MediaItem), device (Device)
OUTPUT: string[]

BEGIN
    compatiblePlatforms ← []

    FOR EACH platform IN media.platforms DO
        // Check if device supports this platform
        IF device.capabilities.contains(platform) THEN
            compatiblePlatforms.append(platform)
        END IF
    END FOR

    RETURN compatiblePlatforms
END
```

**Time Complexity**: O(n × m) where n = media platforms, m = device capabilities
**Space Complexity**: O(1) for deep link generation

---

## 8. AUTH COMMAND

```
ALGORITHM: AuthCommand.execute
INPUT: action (string), options (AuthOptions)
OUTPUT: AuthResult

DATA STRUCTURES:
    AuthActions:
        LOGIN = "login"
        LOGOUT = "logout"
        STATUS = "status"
        REFRESH = "refresh"

    AuthResult:
        authenticated: boolean
        user: User
        expiresAt: timestamp
        tokenType: string

BEGIN
    CASE action OF
        AuthActions.LOGIN:
            RETURN Login(options)

        AuthActions.LOGOUT:
            RETURN Logout(options)

        AuthActions.STATUS:
            RETURN GetAuthStatus(options)

        AuthActions.REFRESH:
            RETURN RefreshAuth(options)

        OTHERWISE:
            RETURN Error("Invalid action: " + action)
    END CASE
END

SUBROUTINE: Login
INPUT: options (AuthOptions)
OUTPUT: AuthResult

BEGIN
    // Check if already authenticated
    IF IsAuthenticated() THEN
        currentUser ← GetCurrentUser()
        logger.info("Already authenticated as: " + currentUser.email)

        confirmed ← PromptConfirmation("Login with different account?")
        IF NOT confirmed THEN
            RETURN Cancelled("Login cancelled")
        END IF

        // Logout first
        Logout({ confirm: false })
    END IF

    // Device authorization flow
    logger.info("Starting device authorization...")
    logger.newline()

    // Request device code
    deviceAuth ← AuthAPI.requestDeviceCode()

    // Display authorization instructions
    logger.box(
        "Device Authorization Required\n\n" +
        "1. Visit: " + chalk.cyan.bold(deviceAuth.verificationUrl) + "\n" +
        "2. Enter code: " + chalk.yellow.bold(deviceAuth.userCode) + "\n\n" +
        "Waiting for authorization...",
        "Login"
    )

    // Poll for authorization
    spinner ← ShowSpinner("Waiting for authorization")
    maxAttempts ← 60  // 5 minutes (5 second intervals)
    attempts ← 0

    WHILE attempts < maxAttempts DO
        Sleep(5000)  // Wait 5 seconds

        pollResult ← AuthAPI.pollDeviceCode(deviceAuth.deviceCode)

        IF pollResult.status = "authorized" THEN
            spinner.stop()

            // Store tokens
            TokenManager.saveTokens(pollResult.tokens)

            // Get user info
            user ← AuthAPI.getUserInfo(pollResult.tokens.accessToken)

            // Save user config
            config.user ← user
            ConfigManager.save(config)

            logger.success("Successfully authenticated as: " + user.email)

            RETURN {
                success: true,
                authenticated: true,
                user: user,
                expiresAt: pollResult.tokens.expiresAt
            }

        ELSE IF pollResult.status = "denied" THEN
            spinner.stop()
            RETURN Error("Authorization denied")

        ELSE IF pollResult.status = "expired" THEN
            spinner.stop()
            RETURN Error("Authorization code expired. Please try again.")
        END IF

        attempts ← attempts + 1
    END WHILE

    spinner.stop()
    RETURN Error("Authorization timeout. Please try again.")
END

SUBROUTINE: Logout
INPUT: options (AuthOptions)
OUTPUT: Result

BEGIN
    IF NOT IsAuthenticated() THEN
        logger.info("Not currently authenticated")
        RETURN { success: true }
    END IF

    currentUser ← GetCurrentUser()

    IF options.confirm THEN
        confirmed ← PromptConfirmation("Logout from account: " + currentUser.email + "?")
        IF NOT confirmed THEN
            RETURN Cancelled("Logout cancelled")
        END IF
    END IF

    // Revoke tokens
    TRY
        tokens ← TokenManager.getTokens()
        AuthAPI.revokeTokens(tokens.accessToken)
    CATCH error AS e
        logger.warn("Failed to revoke tokens: " + e.message)
    END TRY

    // Clear local tokens and config
    TokenManager.clearTokens()
    config.user ← null
    ConfigManager.save(config)

    logger.success("Logged out successfully")

    RETURN { success: true }
END

SUBROUTINE: GetAuthStatus
INPUT: options (AuthOptions)
OUTPUT: AuthStatusResult

BEGIN
    authenticated ← IsAuthenticated()

    IF NOT authenticated THEN
        logger.info("Status: Not authenticated")
        logger.info("Run 'cli auth login' to authenticate")

        RETURN {
            authenticated: false
        }
    END IF

    // Get user and token info
    user ← GetCurrentUser()
    tokens ← TokenManager.getTokens()

    // Check token expiration
    expiresAt ← tokens.expiresAt
    expiresIn ← expiresAt - GetCurrentTime()
    isExpired ← expiresIn <= 0

    // Display status
    logger.info("Status: Authenticated")
    logger.info("User: " + user.email)
    logger.info("User ID: " + user.id)

    IF isExpired THEN
        logger.warn("Token: Expired")
        logger.info("Run 'cli auth refresh' to refresh")
    ELSE
        expiresInHours ← FLOOR(expiresIn / 3600)
        logger.info("Token: Valid (expires in " + expiresInHours + " hours)")
    END IF

    RETURN {
        authenticated: true,
        user: user,
        tokenExpired: isExpired,
        expiresAt: expiresAt
    }
END
```

**Time Complexity**:
- Login: O(1) per poll attempt
- Logout: O(1)
- Status: O(1)

**Space Complexity**: O(1) for token storage

---

## 9. MCP COMMAND

```
ALGORITHM: MCPCommand.execute
INPUT: transport (string), options (MCPOptions)
OUTPUT: ServerResult

DATA STRUCTURES:
    MCPOptions:
        port: integer
        host: string
        configFile: string
        logLevel: LogLevel

    MCPServer:
        transport: Transport
        handlers: Map<string, Handler>
        middleware: Middleware[]

BEGIN
    // Validate transport
    IF transport NOT IN ["stdio", "sse"] THEN
        RETURN Error("Invalid transport. Use 'stdio' or 'sse'")
    END IF

    // Load MCP configuration
    mcpConfig ← LoadMCPConfig(options.configFile)

    CASE transport OF
        "stdio":
            RETURN StartSTDIOServer(mcpConfig, options)

        "sse":
            RETURN StartSSEServer(mcpConfig, options)

        OTHERWISE:
            RETURN Error("Unsupported transport: " + transport)
    END CASE
END

SUBROUTINE: StartSTDIOServer
INPUT: config (MCPConfig), options (MCPOptions)
OUTPUT: ServerResult

BEGIN
    logger.info("Starting MCP server (STDIO transport)...")

    // Initialize server
    server ← NEW MCPServer({
        transport: "stdio",
        config: config
    })

    // Register tool handlers
    RegisterToolHandlers(server)

    // Setup STDIO streams
    server.stdin ← process.stdin
    server.stdout ← process.stdout

    // Message handler
    server.on("message", (message) => {
        HandleMCPMessage(server, message)
    })

    // Error handler
    server.on("error", (error) => {
        LogError("MCP server error: " + error.message)
    })

    // Start listening
    server.listen()

    logger.success("MCP server started on STDIO")
    logger.info("Server ready to receive messages")

    // Keep process running
    WHILE true DO
        Sleep(1000)
    END WHILE
END

SUBROUTINE: StartSSEServer
INPUT: config (MCPConfig), options (MCPOptions)
OUTPUT: ServerResult

BEGIN
    port ← options.port || config.defaultPort || 3000
    host ← options.host || "localhost"

    logger.info("Starting MCP server (SSE transport)...")
    logger.info("Host: " + host)
    logger.info("Port: " + port)

    // Initialize HTTP server
    app ← CreateExpressApp()

    // Middleware
    app.use(Helmet())  // Security headers
    app.use(RateLimit({
        windowMs: 60000,    // 1 minute
        max: 100            // 100 requests per minute
    }))
    app.use(BodyParser.json())

    // CORS
    app.use(CORS({
        origin: config.allowedOrigins || ["*"],
        credentials: true
    }))

    // SSE endpoint
    app.get("/sse", (request, response) => {
        SetupSSEConnection(response, config)
    })

    // Tools endpoint
    app.get("/tools", (request, response) => {
        tools ← GetAvailableTools()
        response.json({ tools: tools })
    })

    // Execute tool endpoint
    app.post("/execute", async (request, response) => {
        TRY
            result ← await ExecuteTool(request.body)
            response.json(result)
        CATCH error AS e
            response.status(500).json({
                error: error.message
            })
        END TRY
    })

    // Health check
    app.get("/health", (request, response) => {
        response.json({
            status: "ok",
            uptime: process.uptime(),
            timestamp: GetCurrentTime()
        })
    })

    // Start server
    server ← app.listen(port, host, () => {
        logger.success("MCP server started")
        logger.info("URL: http://" + host + ":" + port)
        logger.info("SSE endpoint: http://" + host + ":" + port + "/sse")
        logger.divider()
    })

    // Graceful shutdown
    RegisterShutdownHandler(() => {
        logger.info("Shutting down MCP server...")
        server.close()
    })

    RETURN {
        success: true,
        url: "http://" + host + ":" + port
    }
END

SUBROUTINE: HandleMCPMessage
INPUT: server (MCPServer), message (MCPMessage)
OUTPUT: void

BEGIN
    TRY
        // Parse JSON-RPC message
        request ← ParseJSONRPC(message)

        // Validate request
        IF NOT request.jsonrpc = "2.0" THEN
            SendError(server, request.id, -32600, "Invalid JSON-RPC version")
            RETURN
        END IF

        // Get method handler
        handler ← server.handlers.get(request.method)

        IF handler IS null THEN
            SendError(server, request.id, -32601, "Method not found: " + request.method)
            RETURN
        END IF

        // Execute handler
        result ← handler(request.params)

        // Send response
        response ← {
            jsonrpc: "2.0",
            id: request.id,
            result: result
        }

        server.send(JSON.stringify(response))

    CATCH ParseError AS e
        SendError(server, null, -32700, "Parse error: " + e.message)

    CATCH error AS e
        SendError(server, request.id, -32603, "Internal error: " + e.message)
    END TRY
END

SUBROUTINE: RegisterToolHandlers
INPUT: server (MCPServer)
OUTPUT: void

BEGIN
    // Search tool
    server.registerTool("search", (params) => {
        SearchCommand.execute(params.query, params.options)
    })

    // Recommend tool
    server.registerTool("recommend", (params) => {
        RecommendCommand.execute(params.context, params.options)
    })

    // Watchlist tools
    server.registerTool("watchlist.list", (params) => {
        WatchlistCommand.execute("list", params.options)
    })

    server.registerTool("watchlist.add", (params) => {
        WatchlistCommand.execute("add", params)
    })

    server.registerTool("watchlist.remove", (params) => {
        WatchlistCommand.execute("remove", params)
    })

    // Device tools
    server.registerTool("devices.list", (params) => {
        DevicesCommand.execute("list", params.options)
    })

    // Cast tool
    server.registerTool("cast", (params) => {
        CastCommand.execute(params.mediaId, params.options)
    })

    // Auth tools
    server.registerTool("auth.status", (params) => {
        AuthCommand.execute("status", params)
    })
END
```

**Time Complexity**: O(1) for message handling
**Space Complexity**: O(n) where n = number of concurrent SSE connections

---

## 10. OUTPUT FORMATTING

### 10.1 Table Rendering

```
ALGORITHM: FormatTable
INPUT: items (array), options (TableOptions)
OUTPUT: formattedTable (string)

DATA STRUCTURES:
    TableOptions:
        columns: string[]
        columnWidths: integer[]
        style: "ascii" | "rounded" | "minimal"
        headers: boolean
        formatters: Map<string, Formatter>
        maxRows: integer

    TableStyle:
        topLeft: string
        topRight: string
        bottomLeft: string
        bottomRight: string
        horizontal: string
        vertical: string
        cross: string

BEGIN
    // Get style characters
    style ← GetTableStyle(options.style)

    // Calculate column widths if not provided
    IF options.columnWidths IS empty THEN
        columnWidths ← CalculateColumnWidths(items, options.columns)
    ELSE
        columnWidths ← options.columnWidths
    END IF

    output ← []

    // Top border
    topBorder ← BuildBorder(style.topLeft, style.horizontal, style.topRight, columnWidths)
    output.append(topBorder)

    // Headers
    IF options.headers THEN
        headerRow ← BuildRow(options.columns, columnWidths, style.vertical, null)
        output.append(headerRow)

        // Header separator
        separator ← BuildBorder(style.cross, style.horizontal, style.cross, columnWidths)
        output.append(separator)
    END IF

    // Data rows
    maxRows ← options.maxRows || items.length
    FOR i = 0 TO MIN(maxRows, items.length) - 1 DO
        item ← items[i]

        // Extract column values
        values ← []
        FOR EACH column IN options.columns DO
            value ← GetColumnValue(item, column)

            // Apply formatter if exists
            IF options.formatters.has(column) THEN
                formatter ← options.formatters.get(column)
                value ← formatter(value, item)
            END IF

            values.append(value)
        END FOR

        row ← BuildRow(values, columnWidths, style.vertical, options.formatters)
        output.append(row)
    END FOR

    // Bottom border
    bottomBorder ← BuildBorder(style.bottomLeft, style.horizontal, style.bottomRight, columnWidths)
    output.append(bottomBorder)

    // Truncation notice
    IF items.length > maxRows THEN
        output.append("... (" + (items.length - maxRows) + " more rows)")
    END IF

    RETURN output.join("\n")
END

SUBROUTINE: BuildRow
INPUT: values (string[]), widths (integer[]), separator (string), formatters (Map)
OUTPUT: row (string)

BEGIN
    cells ← []

    FOR i = 0 TO values.length - 1 DO
        value ← ToString(values[i])
        width ← widths[i]

        // Truncate if too long
        IF value.length > width THEN
            value ← value.substring(0, width - 3) + "..."
        END IF

        // Pad to width
        cell ← PadRight(value, width)

        cells.append(cell)
    END FOR

    RETURN separator + " " + cells.join(" " + separator + " ") + " " + separator
END

SUBROUTINE: GetTableStyle
INPUT: styleName (string)
OUTPUT: TableStyle

BEGIN
    CASE styleName OF
        "ascii":
            RETURN {
                topLeft: "+",
                topRight: "+",
                bottomLeft: "+",
                bottomRight: "+",
                horizontal: "-",
                vertical: "|",
                cross: "+"
            }

        "rounded":
            RETURN {
                topLeft: "╭",
                topRight: "╮",
                bottomLeft: "╰",
                bottomRight: "╯",
                horizontal: "─",
                vertical: "│",
                cross: "┼"
            }

        "minimal":
            RETURN {
                topLeft: "",
                topRight: "",
                bottomLeft: "",
                bottomRight: "",
                horizontal: "─",
                vertical: " ",
                cross: " "
            }

        OTHERWISE:
            RETURN GetTableStyle("ascii")
    END CASE
END
```

**Time Complexity**: O(r × c) where r = rows, c = columns
**Space Complexity**: O(r × c) for output buffer

### 10.2 JSON Output

```
ALGORITHM: FormatJSON
INPUT: data (any), options (JSONOptions)
OUTPUT: jsonString (string)

DATA STRUCTURES:
    JSONOptions:
        pretty: boolean
        indent: integer
        sortKeys: boolean
        excludeNull: boolean

BEGIN
    IF options.excludeNull THEN
        data ← RemoveNullValues(data)
    END IF

    IF options.sortKeys THEN
        data ← SortObjectKeys(data)
    END IF

    IF options.pretty THEN
        indent ← options.indent || 2
        RETURN JSON.stringify(data, null, indent)
    ELSE
        RETURN JSON.stringify(data)
    END IF
END
```

### 10.3 Progress Indicators

```
ALGORITHM: ShowSpinner
INPUT: message (string)
OUTPUT: Spinner

DATA STRUCTURES:
    Spinner:
        frames: string[]
        interval: integer
        currentFrame: integer
        isRunning: boolean

BEGIN
    spinner ← {
        frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        interval: 80,
        currentFrame: 0,
        isRunning: true,
        message: message
    }

    // Start animation
    timerId ← SetInterval(() => {
        IF spinner.isRunning THEN
            frame ← spinner.frames[spinner.currentFrame]
            output ← "\r" + frame + " " + spinner.message

            process.stdout.write(output)

            spinner.currentFrame ← (spinner.currentFrame + 1) % spinner.frames.length
        END IF
    }, spinner.interval)

    // Stop method
    spinner.stop ← () => {
        spinner.isRunning ← false
        ClearInterval(timerId)
        process.stdout.write("\r" + " ".repeat(spinner.message.length + 5) + "\r")
    }

    RETURN spinner
END

ALGORITHM: ShowProgressBar
INPUT: current (integer), total (integer), options (ProgressOptions)
OUTPUT: void

BEGIN
    width ← options.width || 40
    percentage ← (current / total) * 100
    filled ← FLOOR((current / total) * width)
    empty ← width - filled

    bar ← "█".repeat(filled) + "░".repeat(empty)

    output ← "\r[" + bar + "] " +
             FLOOR(percentage) + "% " +
             "(" + current + "/" + total + ")"

    process.stdout.write(output)

    IF current = total THEN
        process.stdout.write("\n")
    END IF
END
```

---

## 11. ERROR HANDLING AND EXIT CODES

```
EXIT CODES:
    SUCCESS = 0                 // Operation completed successfully
    ERROR = 1                   // General error
    INVALID_ARGS = 2           // Invalid command line arguments
    AUTH_REQUIRED = 3          // Authentication required
    NETWORK_ERROR = 4          // Network/API error
    PERMISSION_DENIED = 5      // Permission denied
    NOT_FOUND = 6              // Resource not found
    TIMEOUT = 7                // Operation timeout
    CANCELLED = 8              // User cancelled operation

ERROR MESSAGES:
    Map of error codes to user-friendly messages

    INVALID_COMMAND: "Unknown command: '{command}'. Run 'cli help' for available commands."
    MISSING_ARGUMENT: "Missing required argument: {argument}"
    INVALID_OPTION: "Invalid option: {option}"
    AUTH_REQUIRED: "This command requires authentication. Run 'cli auth login' to continue."
    NETWORK_ERROR: "Network error: {details}. Check your internet connection."
    API_ERROR: "API error: {details}. Please try again later."
    NOT_FOUND: "{resource} not found: {id}"
    TIMEOUT: "Operation timed out after {seconds} seconds."
```

---

## 12. HELP TEXT GENERATION

```
ALGORITHM: GenerateHelp
INPUT: command (string | null)
OUTPUT: helpText (string)

BEGIN
    IF command IS null THEN
        RETURN GenerateGlobalHelp()
    ELSE
        RETURN GenerateCommandHelp(command)
    END IF
END

SUBROUTINE: GenerateGlobalHelp
OUTPUT: helpText (string)

BEGIN
    output ← []

    output.append("Media Gateway CLI v" + version)
    output.append("")
    output.append("USAGE:")
    output.append("  cli <command> [options]")
    output.append("")
    output.append("COMMANDS:")

    commands ← [
        { name: "search", desc: "Search for movies, TV shows, and music" },
        { name: "recommend", desc: "Get personalized recommendations" },
        { name: "watchlist", desc: "Manage your watchlist" },
        { name: "devices", desc: "Manage registered devices" },
        { name: "cast", desc: "Cast media to a device" },
        { name: "auth", desc: "Authentication management" },
        { name: "mcp", desc: "Start MCP server" },
        { name: "config", desc: "Configuration management" },
        { name: "help", desc: "Show help information" }
    ]

    maxNameLength ← MaxLength(commands.map(c => c.name))

    FOR EACH cmd IN commands DO
        padding ← " ".repeat(maxNameLength - cmd.name.length + 2)
        output.append("  " + cmd.name + padding + cmd.desc)
    END FOR

    output.append("")
    output.append("OPTIONS:")
    output.append("  --help, -h      Show help")
    output.append("  --version, -v   Show version")
    output.append("  --json          Output as JSON")
    output.append("  --quiet, -q     Suppress non-essential output")
    output.append("")
    output.append("EXAMPLES:")
    output.append("  cli search \"inception\" --type movie")
    output.append("  cli recommend --mood action --context \"date night\"")
    output.append("  cli watchlist add <media-id>")
    output.append("  cli cast <media-id> --device <device-id>")
    output.append("")
    output.append("For command-specific help: cli help <command>")

    RETURN output.join("\n")
END

SUBROUTINE: GenerateCommandHelp
INPUT: commandName (string)
OUTPUT: helpText (string)

BEGIN
    command ← commands.get(commandName)

    IF command IS null THEN
        RETURN "Unknown command: " + commandName
    END IF

    output ← []

    output.append(command.name.toUpperCase() + " - " + command.description)
    output.append("")
    output.append("USAGE:")
    output.append("  " + command.usage)
    output.append("")

    IF command.options.length > 0 THEN
        output.append("OPTIONS:")
        FOR EACH option IN command.options DO
            optionStr ← "  " + option.flags.join(", ")
            padding ← " ".repeat(30 - optionStr.length)
            output.append(optionStr + padding + option.description)

            IF option.default IS NOT null THEN
                output.append(" ".repeat(32) + "(default: " + option.default + ")")
            END IF
        END FOR
        output.append("")
    END IF

    IF command.examples.length > 0 THEN
        output.append("EXAMPLES:")
        FOR EACH example IN command.examples DO
            output.append("  " + example)
        END FOR
        output.append("")
    END IF

    RETURN output.join("\n")
END
```

---

## 13. COMPLEXITY ANALYSIS SUMMARY

### Command Execution Complexity

| Command | Time Complexity | Space Complexity | Notes |
|---------|----------------|------------------|-------|
| search | O(n log n) | O(r) | n = result set, r = cached results |
| recommend | O(n log n) | O(n) | Sorting recommendations |
| watchlist list | O(1) | O(p) | p = page size (constant) |
| watchlist add | O(1) | O(1) | Single item operation |
| watchlist sync | O(n) | O(n) | n = watchlist size |
| devices list | O(n) | O(n) | n = number of devices |
| cast | O(p × c) | O(1) | p = platforms, c = capabilities |
| auth login | O(t) | O(1) | t = polling attempts |
| mcp start | O(1) | O(c) | c = concurrent connections |

### Parsing and Formatting

| Operation | Time Complexity | Space Complexity |
|-----------|----------------|------------------|
| Argument parsing | O(a) | O(a) | a = number of arguments |
| Table formatting | O(r × c) | O(r × c) | r = rows, c = columns |
| JSON formatting | O(n) | O(n) | n = data size |
| Help generation | O(c) | O(c) | c = number of commands |

---

## 14. DESIGN PATTERNS

### 14.1 Command Pattern
```
INTERFACE: Command
    execute(options): Result
    validate(options): ValidationResult
    requiresAuth(): boolean

Each CLI command implements this interface
```

### 14.2 Strategy Pattern
```
INTERFACE: OutputFormatter
    format(data, options): string

Implementations:
    - TableFormatter
    - JSONFormatter
    - PrettyFormatter
```

### 14.3 Observer Pattern
```
CLASS: EventEmitter
    Used for:
    - Progress updates
    - Server events
    - Authentication flow updates
```

---

## END OF PSEUDOCODE SPECIFICATION

This pseudocode provides a complete algorithmic blueprint for implementing the Media Gateway CLI, focusing on:
- Clear, language-agnostic logic
- Comprehensive error handling
- Optimal data structures
- Complexity analysis
- Interactive and batch modes
- Extensible architecture

Ready for implementation in any programming language.
