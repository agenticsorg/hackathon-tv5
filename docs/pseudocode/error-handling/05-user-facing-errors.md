# User-Facing Error Messages - Pseudocode

## Overview
Error message mapping, localization, actionable suggestions, and support escalation for optimal user experience.

---

## Data Structures

```
DATA STRUCTURE: UserErrorMessage
    code: string
    title: string
    message: string
    description: string
    severity: INFO | WARNING | ERROR | CRITICAL
    suggestions: array of Suggestion
    actions: array of Action
    supportInfo: SupportInfo
    localeKey: string
    category: string

DATA STRUCTURE: Suggestion
    text: string
    icon: string
    priority: integer
    actionable: boolean
    action: function or null

DATA STRUCTURE: Action
    label: string
    type: PRIMARY | SECONDARY | TERTIARY
    handler: function
    requiresAuth: boolean
    dangerous: boolean

DATA STRUCTURE: SupportInfo
    showContactSupport: boolean
    errorReference: string
    timestamp: timestamp
    context: object
    escalationLevel: LOW | MEDIUM | HIGH | CRITICAL
    supportChannel: EMAIL | CHAT | PHONE | TICKET
```

---

## Algorithm: Get User Error Message

```
ALGORITHM: GetUserErrorMessage
INPUT: error (Error), context (object), locale (string)
OUTPUT: userMessage (UserErrorMessage)

BEGIN
    // Classify error
    errorType ← ClassifyError(error, context)
    errorMetadata ← GetErrorMetadata(errorType)

    // Get base message template
    messageTemplate ← GetMessageTemplate(errorType, locale)

    // Build user-facing message
    userMessage ← UserErrorMessage()
    userMessage.code ← errorMetadata.code
    userMessage.severity ← errorMetadata.severity
    userMessage.category ← GetErrorCategory(errorType)

    // Localize message
    userMessage.title ← LocalizeString(
        messageTemplate.titleKey,
        locale,
        context
    )
    userMessage.message ← LocalizeString(
        messageTemplate.messageKey,
        locale,
        context
    )
    userMessage.description ← LocalizeString(
        messageTemplate.descriptionKey,
        locale,
        context
    )

    // Generate suggestions
    userMessage.suggestions ← GenerateSuggestions(
        errorType,
        context,
        locale
    )

    // Generate actions
    userMessage.actions ← GenerateActions(
        errorType,
        context,
        locale
    )

    // Add support information
    userMessage.supportInfo ← GenerateSupportInfo(
        error,
        errorType,
        context
    )

    // Log user-facing error
    LogUserError(userMessage, error, context)

    RETURN userMessage
END
```

---

## Error Message Templates

```
DATA: ErrorMessageTemplates

// Network Errors
NetworkError.ConnectionTimeout:
    titleKey: "error.network.timeout.title"
    messageKey: "error.network.timeout.message"
    descriptionKey: "error.network.timeout.description"
    defaultTitle: "Connection Timed Out"
    defaultMessage: "We couldn't connect to the service in time"
    defaultDescription: "This might be due to slow internet or server issues"

NetworkError.DNSFailure:
    titleKey: "error.network.dns.title"
    messageKey: "error.network.dns.message"
    descriptionKey: "error.network.dns.description"
    defaultTitle: "Unable to Reach Server"
    defaultMessage: "We couldn't find the server"
    defaultDescription: "Please check your internet connection"

NetworkError.SSLError:
    titleKey: "error.network.ssl.title"
    messageKey: "error.network.ssl.message"
    descriptionKey: "error.network.ssl.description"
    defaultTitle: "Secure Connection Failed"
    defaultMessage: "We couldn't establish a secure connection"
    defaultDescription: "This might be due to network security settings"

// Authentication Errors
AuthenticationError.TokenExpired:
    titleKey: "error.auth.expired.title"
    messageKey: "error.auth.expired.message"
    descriptionKey: "error.auth.expired.description"
    defaultTitle: "Session Expired"
    defaultMessage: "Your session has expired"
    defaultDescription: "Please sign in again to continue"

AuthenticationError.InsufficientScope:
    titleKey: "error.auth.scope.title"
    messageKey: "error.auth.scope.message"
    descriptionKey: "error.auth.scope.description"
    defaultTitle: "Additional Permissions Required"
    defaultMessage: "This action requires additional permissions"
    defaultDescription: "Please grant the necessary permissions to continue"

// API Errors
APIError.RateLimitExceeded:
    titleKey: "error.api.ratelimit.title"
    messageKey: "error.api.ratelimit.message"
    descriptionKey: "error.api.ratelimit.description"
    defaultTitle: "Too Many Requests"
    defaultMessage: "You've made too many requests"
    defaultDescription: "Please wait a moment before trying again"

APIError.ServiceUnavailable:
    titleKey: "error.api.unavailable.title"
    messageKey: "error.api.unavailable.message"
    descriptionKey: "error.api.unavailable.description"
    defaultTitle: "Service Temporarily Unavailable"
    defaultMessage: "The service is currently unavailable"
    defaultDescription: "We're working to restore it. Please try again later"

// Platform Errors
PlatformError.YouTubeQuotaExceeded:
    titleKey: "error.platform.youtube.quota.title"
    messageKey: "error.platform.youtube.quota.message"
    descriptionKey: "error.platform.youtube.quota.description"
    defaultTitle: "YouTube API Limit Reached"
    defaultMessage: "We've reached our YouTube API limit for today"
    defaultDescription: "Showing cached content until the limit resets"

PlatformError.RegionRestricted:
    titleKey: "error.platform.region.title"
    messageKey: "error.platform.region.message"
    descriptionKey: "error.platform.region.description"
    defaultTitle: "Content Not Available"
    defaultMessage: "This content isn't available in your region"
    defaultDescription: "Geographic restrictions prevent access to this content"

// Data Errors
DataError.SyncConflict:
    titleKey: "error.data.conflict.title"
    messageKey: "error.data.conflict.message"
    descriptionKey: "error.data.conflict.description"
    defaultTitle: "Sync Conflict Detected"
    defaultMessage: "Your data conflicts with the server version"
    defaultDescription: "We're automatically resolving the conflict"
```

---

## Algorithm: Generate Suggestions

```
ALGORITHM: GenerateSuggestions
INPUT: errorType (ErrorType), context (object), locale (string)
OUTPUT: suggestions (array of Suggestion)

BEGIN
    suggestions ← []

    SWITCH errorType:
        CASE NetworkError.ConnectionTimeout:
            suggestions.append(Suggestion(
                text: Localize("suggestion.check_connection", locale),
                icon: "wifi",
                priority: 1,
                actionable: true,
                action: OpenNetworkSettings
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.retry_later", locale),
                icon: "refresh",
                priority: 2,
                actionable: false,
                action: null
            ))

        CASE NetworkError.SSLError:
            suggestions.append(Suggestion(
                text: Localize("suggestion.check_time", locale),
                icon: "clock",
                priority: 1,
                actionable: true,
                action: OpenDateTimeSettings
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.check_antivirus", locale),
                icon: "shield",
                priority: 2,
                actionable: false,
                action: null
            ))

        CASE AuthenticationError.TokenExpired:
            suggestions.append(Suggestion(
                text: Localize("suggestion.sign_in_again", locale),
                icon: "login",
                priority: 1,
                actionable: true,
                action: InitiateReauthentication
            ))

        CASE AuthenticationError.InsufficientScope:
            suggestions.append(Suggestion(
                text: Localize("suggestion.grant_permissions", locale),
                icon: "key",
                priority: 1,
                actionable: true,
                action: ShowPermissionDialog
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.upgrade_account", locale),
                icon: "upgrade",
                priority: 2,
                actionable: true,
                action: ShowUpgradeOptions
            ))

        CASE APIError.RateLimitExceeded:
            retryAfter ← GetRetryAfterSeconds(context.error)
            suggestions.append(Suggestion(
                text: Localize("suggestion.wait_retry", locale, {
                    seconds: retryAfter
                }),
                icon: "timer",
                priority: 1,
                actionable: false,
                action: null
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.use_cached", locale),
                icon: "storage",
                priority: 2,
                actionable: true,
                action: ShowCachedContent
            ))

        CASE APIError.ServiceUnavailable:
            suggestions.append(Suggestion(
                text: Localize("suggestion.check_status", locale),
                icon: "info",
                priority: 1,
                actionable: true,
                action: OpenStatusPage
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.offline_mode", locale),
                icon: "offline",
                priority: 2,
                actionable: true,
                action: ActivateOfflineMode
            ))

        CASE PlatformError.YouTubeQuotaExceeded:
            suggestions.append(Suggestion(
                text: Localize("suggestion.cached_content", locale),
                icon: "storage",
                priority: 1,
                actionable: true,
                action: ShowCachedContent
            ))
            resetTime ← GetQuotaResetTime()
            suggestions.append(Suggestion(
                text: Localize("suggestion.quota_reset", locale, {
                    time: FormatTime(resetTime, locale)
                }),
                icon: "schedule",
                priority: 2,
                actionable: false,
                action: null
            ))

        CASE PlatformError.RegionRestricted:
            suggestions.append(Suggestion(
                text: Localize("suggestion.find_alternatives", locale),
                icon: "search",
                priority: 1,
                actionable: true,
                action: FindAlternativeContent
            ))

        CASE DataError.SyncConflict:
            suggestions.append(Suggestion(
                text: Localize("suggestion.auto_resolving", locale),
                icon: "sync",
                priority: 1,
                actionable: false,
                action: null
            ))
            suggestions.append(Suggestion(
                text: Localize("suggestion.view_changes", locale),
                icon: "compare",
                priority: 2,
                actionable: true,
                action: ShowConflictDetails
            ))

        DEFAULT:
            suggestions.append(Suggestion(
                text: Localize("suggestion.retry", locale),
                icon: "refresh",
                priority: 1,
                actionable: true,
                action: RetryLastOperation
            ))
    END SWITCH

    // Sort by priority
    suggestions.sort(BY priority ASCENDING)

    RETURN suggestions
END
```

---

## Algorithm: Generate Actions

```
ALGORITHM: GenerateActions
INPUT: errorType (ErrorType), context (object), locale (string)
OUTPUT: actions (array of Action)

BEGIN
    actions ← []

    SWITCH errorType:
        CASE NetworkError.ConnectionTimeout:
        CASE NetworkError.DNSFailure:
            actions.append(Action(
                label: Localize("action.retry", locale),
                type: PRIMARY,
                handler: RetryOperation,
                requiresAuth: false,
                dangerous: false
            ))
            actions.append(Action(
                label: Localize("action.use_cached", locale),
                type: SECONDARY,
                handler: UseCachedData,
                requiresAuth: false,
                dangerous: false
            ))

        CASE NetworkError.SSLError:
            actions.append(Action(
                label: Localize("action.help", locale),
                type: PRIMARY,
                handler: ShowSSLHelp,
                requiresAuth: false,
                dangerous: false
            ))

        CASE AuthenticationError.TokenExpired:
            actions.append(Action(
                label: Localize("action.sign_in", locale),
                type: PRIMARY,
                handler: InitiateReauthentication,
                requiresAuth: false,
                dangerous: false
            ))
            actions.append(Action(
                label: Localize("action.cancel", locale),
                type: SECONDARY,
                handler: CancelOperation,
                requiresAuth: false,
                dangerous: false
            ))

        CASE AuthenticationError.InsufficientScope:
            actions.append(Action(
                label: Localize("action.grant_access", locale),
                type: PRIMARY,
                handler: ShowPermissionDialog,
                requiresAuth: true,
                dangerous: false
            ))
            actions.append(Action(
                label: Localize("action.upgrade", locale),
                type: SECONDARY,
                handler: ShowUpgradeOptions,
                requiresAuth: true,
                dangerous: false
            ))

        CASE APIError.RateLimitExceeded:
            actions.append(Action(
                label: Localize("action.wait_retry", locale),
                type: PRIMARY,
                handler: WaitAndRetry,
                requiresAuth: false,
                dangerous: false
            ))

        CASE APIError.ServiceUnavailable:
            actions.append(Action(
                label: Localize("action.retry", locale),
                type: PRIMARY,
                handler: RetryOperation,
                requiresAuth: false,
                dangerous: false
            ))
            actions.append(Action(
                label: Localize("action.offline_mode", locale),
                type: SECONDARY,
                handler: ActivateOfflineMode,
                requiresAuth: false,
                dangerous: false
            ))

        CASE PlatformError.YouTubeQuotaExceeded:
            actions.append(Action(
                label: Localize("action.view_cached", locale),
                type: PRIMARY,
                handler: ShowCachedContent,
                requiresAuth: false,
                dangerous: false
            ))

        CASE PlatformError.RegionRestricted:
            actions.append(Action(
                label: Localize("action.find_similar", locale),
                type: PRIMARY,
                handler: FindAlternativeContent,
                requiresAuth: false,
                dangerous: false
            ))

        CASE DataError.SyncConflict:
            actions.append(Action(
                label: Localize("action.view_details", locale),
                type: PRIMARY,
                handler: ShowConflictDetails,
                requiresAuth: true,
                dangerous: false
            ))
            actions.append(Action(
                label: Localize("action.accept_server", locale),
                type: SECONDARY,
                handler: AcceptServerVersion,
                requiresAuth: true,
                dangerous: true
            ))
            actions.append(Action(
                label: Localize("action.keep_local", locale),
                type: TERTIARY,
                handler: KeepLocalVersion,
                requiresAuth: true,
                dangerous: true
            ))

        DEFAULT:
            actions.append(Action(
                label: Localize("action.dismiss", locale),
                type: PRIMARY,
                handler: DismissError,
                requiresAuth: false,
                dangerous: false
            ))
    END SWITCH

    RETURN actions
END
```

---

## Algorithm: Generate Support Info

```
ALGORITHM: GenerateSupportInfo
INPUT: error (Error), errorType (ErrorType), context (object)
OUTPUT: supportInfo (SupportInfo)

BEGIN
    supportInfo ← SupportInfo()

    // Generate unique error reference
    supportInfo.errorReference ← GenerateErrorReference(error, context)
    supportInfo.timestamp ← GetCurrentTime()

    // Sanitize context (remove sensitive data)
    supportInfo.context ← SanitizeContext(context)

    // Determine escalation level
    metadata ← GetErrorMetadata(errorType)

    SWITCH metadata.severity:
        CASE CRITICAL:
            supportInfo.escalationLevel ← CRITICAL
            supportInfo.showContactSupport ← true
            supportInfo.supportChannel ← PHONE

        CASE HIGH:
            supportInfo.escalationLevel ← HIGH
            supportInfo.showContactSupport ← true
            supportInfo.supportChannel ← CHAT

        CASE MEDIUM:
            supportInfo.escalationLevel ← MEDIUM
            supportInfo.showContactSupport ← true
            supportInfo.supportChannel ← TICKET

        CASE LOW:
            supportInfo.escalationLevel ← LOW
            supportInfo.showContactSupport ← false
            supportInfo.supportChannel ← EMAIL
    END SWITCH

    // Add additional context for support
    IF supportInfo.showContactSupport THEN
        supportInfo.context.userAgent ← GetUserAgent()
        supportInfo.context.platform ← GetPlatform()
        supportInfo.context.appVersion ← GetAppVersion()
        supportInfo.context.userId ← GetUserId()  // Hashed
        supportInfo.context.sessionId ← GetSessionId()
    END IF

    RETURN supportInfo
END
```

---

## Algorithm: Generate Error Reference

```
ALGORITHM: GenerateErrorReference
INPUT: error (Error), context (object)
OUTPUT: reference (string)

BEGIN
    // Create unique identifier
    timestamp ← GetCurrentTime()
    errorHash ← Hash(error.stack + error.message)
    contextHash ← Hash(JSON.stringify(context))
    randomSuffix ← GenerateRandomString(4)

    // Format: ERR-YYYYMMDD-HASH-SUFFIX
    year ← FormatDate(timestamp, "YYYY")
    month ← FormatDate(timestamp, "MM")
    day ← FormatDate(timestamp, "DD")

    reference ← "ERR-" + year + month + day + "-" +
                errorHash.substring(0, 8) + "-" +
                randomSuffix.toUpperCase()

    // Store reference mapping for support lookup
    StoreErrorReference(reference, {
        error: error,
        context: context,
        timestamp: timestamp
    })

    RETURN reference
END
```

---

## Localization Support

```
ALGORITHM: LocalizeString
INPUT: key (string), locale (string), params (object)
OUTPUT: localizedString (string)

BEGIN
    // Load locale bundle
    localeBundle ← LoadLocaleBundle(locale)

    // Fallback to default locale if key not found
    IF NOT localeBundle.has(key) THEN
        LogWarning("Missing localization key", {
            key: key,
            locale: locale
        })
        localeBundle ← LoadLocaleBundle("en-US")  // Default
    END IF

    // Get template string
    template ← localeBundle.get(key)

    // Replace parameters
    IF params != null THEN
        FOR EACH paramKey IN params.keys() DO
            placeholder ← "{{" + paramKey + "}}"
            template ← template.replace(placeholder, params[paramKey])
        END FOR
    END IF

    RETURN template
END
```

---

## Complexity Analysis

**GetUserErrorMessage:**
- Time Complexity: O(1) - direct lookups and template formatting
- Space Complexity: O(n) where n = number of suggestions/actions

**Generate Suggestions/Actions:**
- Time Complexity: O(1) - switch statement with fixed outputs
- Space Complexity: O(1) - fixed number of suggestions/actions

---

## Design Patterns

1. **Strategy Pattern**: Different message strategies per error type
2. **Template Method**: Message template with localization
3. **Factory Pattern**: Generating suggestions and actions
4. **Builder Pattern**: Constructing complex error messages
