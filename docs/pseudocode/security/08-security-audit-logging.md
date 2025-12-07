# Security Audit Logging - Pseudocode Specification

## Overview
Comprehensive security audit logging system for compliance, forensics, and threat detection. Implements structured logging with severity levels, retention policies, and real-time alerting.

---

## Data Structures

```
STRUCTURE AuditLogEntry:
    id: UUID
    timestamp: Timestamp (microsecond precision)
    event: String (event type identifier)
    severity: Enum["debug", "info", "warning", "error", "critical"]
    user_id: String (null for anonymous)
    ip_address: String
    user_agent: String
    request_id: String (for request correlation)
    resource_type: String (e.g., "user", "video", "token")
    resource_id: String
    action: String (e.g., "create", "read", "update", "delete")
    outcome: Enum["success", "failure", "blocked"]
    details: Object (event-specific data)
    metadata: Object (additional context)
    session_id: String
    client_id: String (for OAuth requests)

STRUCTURE AuditLogQuery:
    start_time: Timestamp
    end_time: Timestamp
    user_id: String (optional)
    event_types: Array<String> (optional)
    severity: Array<String> (optional)
    outcome: String (optional)
    limit: Integer
    offset: Integer

STRUCTURE SecurityAlert:
    id: UUID
    triggered_at: Timestamp
    alert_type: String
    severity: Enum["low", "medium", "high", "critical"]
    description: String
    related_events: Array<UUID> (audit log entry IDs)
    user_id: String
    ip_address: String
    status: Enum["open", "investigating", "resolved", "false_positive"]
    assigned_to: String (security analyst user_id)
```

---

## Algorithm 1: Record Audit Event

```
ALGORITHM: RecordAuditEvent
INPUT: event (string), severity (string), details (object)
OUTPUT: log_entry_id (UUID)

CONSTANTS:
    BATCH_SIZE = 100 // Batch writes for performance
    FLUSH_INTERVAL = 1000 milliseconds

BEGIN
    // Step 1: Extract context from current request
    context ← GetRequestContext()

    // Step 2: Create audit log entry
    log_entry ← AuditLogEntry{
        id: GenerateUUID(),
        timestamp: GetCurrentTimestampMicro(),
        event: event,
        severity: severity,
        user_id: context.user_id,
        ip_address: context.ip_address,
        user_agent: context.user_agent,
        request_id: context.request_id,
        resource_type: details.resource_type,
        resource_id: details.resource_id,
        action: details.action,
        outcome: details.outcome,
        details: details,
        metadata: {
            server_id: GetServerID(),
            version: GetApplicationVersion(),
            environment: GetEnvironment()
        },
        session_id: context.session_id,
        client_id: context.client_id
    }

    // Step 3: Add to write buffer (for batching)
    LogBuffer.append(log_entry)

    // Step 4: Flush if buffer full or critical event
    IF LogBuffer.size() >= BATCH_SIZE OR severity == "critical" THEN
        FlushLogBuffer()
    END IF

    // Step 5: Check for security alerts (async)
    IF severity IN ["error", "critical"] THEN
        CheckSecurityAlerts(log_entry)
    END IF

    // Step 6: Send to real-time monitoring (async)
    IF severity IN ["warning", "error", "critical"] THEN
        PublishToMonitoring(log_entry)
    END IF

    RETURN log_entry.id
END

SUBROUTINE: FlushLogBuffer
OUTPUT: flushed_count (integer)

BEGIN
    // Step 1: Get buffered entries
    entries ← LogBuffer.getAll()

    IF entries is empty THEN
        RETURN 0
    END IF

    // Step 2: Batch insert to database
    TRY:
        Database.batchInsert("audit_logs", entries)
    CATCH exception:
        // Fallback: Write to file system if DB unavailable
        WriteToEmergencyLog(entries, exception)
    END TRY

    // Step 3: Clear buffer
    LogBuffer.clear()

    RETURN entries.length
END
```

**Time Complexity**: O(1) amortized with batching
**Space Complexity**: O(n) where n = buffer size

---

## Algorithm 2: Query Audit Logs

```
ALGORITHM: QueryAuditLogs
INPUT: query (AuditLogQuery)
OUTPUT: results (array of AuditLogEntry)

CONSTANTS:
    MAX_QUERY_RANGE = 2592000 seconds (30 days)
    MAX_RESULTS = 10000

BEGIN
    // Step 1: Validate query parameters
    IF query.end_time - query.start_time > MAX_QUERY_RANGE THEN
        RETURN error("Query range exceeds maximum (30 days)")
    END IF

    IF query.limit > MAX_RESULTS THEN
        query.limit ← MAX_RESULTS
    END IF

    // Step 2: Build database query
    db_query ← {
        timestamp: {
            $gte: query.start_time,
            $lte: query.end_time
        }
    }

    IF query.user_id is not null THEN
        db_query.user_id ← query.user_id
    END IF

    IF query.event_types is not empty THEN
        db_query.event ← {$in: query.event_types}
    END IF

    IF query.severity is not empty THEN
        db_query.severity ← {$in: query.severity}
    END IF

    IF query.outcome is not null THEN
        db_query.outcome ← query.outcome
    END IF

    // Step 3: Execute query
    results ← Database.find(
        collection="audit_logs",
        filter=db_query,
        sort={timestamp: -1}, // Most recent first
        limit=query.limit,
        skip=query.offset
    )

    // Step 4: Audit the audit query (meta-logging)
    RecordAuditEvent(
        event="audit_log_queried",
        severity="info",
        details={
            query_range: query.end_time - query.start_time,
            result_count: results.length,
            queried_by: GetCurrentUserID()
        }
    )

    RETURN results
END
```

**Time Complexity**: O(log n + k) where n = total logs, k = results returned
**Space Complexity**: O(k)

---

## Algorithm 3: Detect Security Anomalies

```
ALGORITHM: CheckSecurityAlerts
INPUT: log_entry (AuditLogEntry)
OUTPUT: alerts_triggered (array of SecurityAlert)

BEGIN
    alerts ← []

    // Pattern 1: Multiple failed login attempts
    IF log_entry.event == "authentication_failed" THEN
        alert ← CheckBruteForceAttempt(log_entry)
        IF alert is not null THEN
            alerts.append(alert)
        END IF
    END IF

    // Pattern 2: Token reuse detection
    IF log_entry.event == "refresh_token_reuse_detected" THEN
        alert ← CreateCriticalAlert(
            type="token_theft_suspected",
            description="Refresh token reuse detected for user " + log_entry.user_id,
            log_entry=log_entry,
            severity="critical"
        )
        alerts.append(alert)
    END IF

    // Pattern 3: Rapid role escalation attempts
    IF log_entry.event == "unauthorized_role_assignment" THEN
        alert ← CheckPrivilegeEscalation(log_entry)
        IF alert is not null THEN
            alerts.append(alert)
        END IF
    END IF

    // Pattern 4: Geographic anomaly
    IF log_entry.user_id is not null THEN
        alert ← CheckGeographicAnomaly(log_entry)
        IF alert is not null THEN
            alerts.append(alert)
        END IF
    END IF

    // Pattern 5: High-volume API abuse
    IF log_entry.event == "rate_limit_exceeded" THEN
        alert ← CheckAPIAbuse(log_entry)
        IF alert is not null THEN
            alerts.append(alert)
        END IF
    END IF

    // Step: Store and notify for each alert
    FOR EACH alert IN alerts DO
        Database.insert("security_alerts", alert)

        // Send real-time notification
        NotifySecurityTeam(alert)

        // Auto-block if critical
        IF alert.severity == "critical" THEN
            ExecuteSecurityAction(alert)
        END IF
    END FOR

    RETURN alerts
END

SUBROUTINE: CheckBruteForceAttempt
INPUT: log_entry (AuditLogEntry)
OUTPUT: alert (SecurityAlert) or null

CONSTANTS:
    FAILURE_THRESHOLD = 5
    TIME_WINDOW = 300 seconds (5 minutes)

BEGIN
    // Count recent failed attempts from same IP
    recent_failures ← Database.count("audit_logs", {
        event: "authentication_failed",
        ip_address: log_entry.ip_address,
        timestamp: {$gte: GetCurrentTimestamp() - TIME_WINDOW}
    })

    IF recent_failures >= FAILURE_THRESHOLD THEN
        // Find all related log entries
        related_events ← Database.find("audit_logs", {
            event: "authentication_failed",
            ip_address: log_entry.ip_address,
            timestamp: {$gte: GetCurrentTimestamp() - TIME_WINDOW}
        }).map(e => e.id)

        alert ← SecurityAlert{
            id: GenerateUUID(),
            triggered_at: GetCurrentTimestamp(),
            alert_type: "brute_force_attempt",
            severity: "high",
            description: recent_failures + " failed login attempts from IP " + log_entry.ip_address,
            related_events: related_events,
            user_id: log_entry.user_id,
            ip_address: log_entry.ip_address,
            status: "open",
            assigned_to: null
        }

        RETURN alert
    END IF

    RETURN null
END

SUBROUTINE: CheckGeographicAnomaly
INPUT: log_entry (AuditLogEntry)
OUTPUT: alert (SecurityAlert) or null

CONSTANTS:
    IMPOSSIBLE_TRAVEL_SPEED = 800 // km/h (faster than commercial flight)

BEGIN
    // Get user's last login location
    last_login ← Database.findOne("audit_logs", {
        event: "authentication_success",
        user_id: log_entry.user_id,
        timestamp: {$lt: log_entry.timestamp}
    }).sort({timestamp: -1})

    IF last_login is null THEN
        RETURN null // No previous login to compare
    END IF

    // Get geographic coordinates
    current_location ← GeoIP.lookup(log_entry.ip_address)
    previous_location ← GeoIP.lookup(last_login.ip_address)

    // Calculate distance and time
    distance_km ← CalculateDistance(
        previous_location.latitude,
        previous_location.longitude,
        current_location.latitude,
        current_location.longitude
    )

    time_elapsed_hours ← (log_entry.timestamp - last_login.timestamp) / 3600

    travel_speed ← distance_km / time_elapsed_hours

    // Check for impossible travel
    IF travel_speed > IMPOSSIBLE_TRAVEL_SPEED THEN
        alert ← SecurityAlert{
            id: GenerateUUID(),
            triggered_at: GetCurrentTimestamp(),
            alert_type: "impossible_travel",
            severity: "high",
            description: "User traveled " + distance_km + " km in " + time_elapsed_hours + " hours",
            related_events: [last_login.id, log_entry.id],
            user_id: log_entry.user_id,
            ip_address: log_entry.ip_address,
            status: "open",
            assigned_to: null
        }

        RETURN alert
    END IF

    RETURN null
END
```

**Time Complexity**: O(log n) for database queries
**Space Complexity**: O(k) where k = number of related events

---

## Algorithm 4: Compliance Reporting

```
ALGORITHM: GenerateComplianceReport
INPUT: start_date (timestamp), end_date (timestamp), report_type (string)
OUTPUT: report (object)

BEGIN
    report ← {
        report_type: report_type,
        generated_at: GetCurrentTimestamp(),
        period: {
            start: start_date,
            end: end_date
        },
        metrics: {},
        events: []
    }

    SWITCH report_type:
        CASE "gdpr_data_access":
            // Report all data access events
            report.events ← Database.find("audit_logs", {
                timestamp: {$gte: start_date, $lte: end_date},
                event: {$in: [
                    "user_data_accessed",
                    "user_data_exported",
                    "user_data_deleted"
                ]}
            })

            report.metrics ← {
                total_access_events: report.events.length,
                unique_users_accessed: CountUnique(report.events, "user_id"),
                data_export_requests: CountByEvent(report.events, "user_data_exported"),
                data_deletion_requests: CountByEvent(report.events, "user_data_deleted")
            }

        CASE "sox_authentication":
            // Report all authentication and authorization events
            report.events ← Database.find("audit_logs", {
                timestamp: {$gte: start_date, $lte: end_date},
                event: {$regex: "^(authentication|authorization)"}
            })

            report.metrics ← {
                total_auth_events: report.events.length,
                successful_logins: CountByEvent(report.events, "authentication_success"),
                failed_logins: CountByEvent(report.events, "authentication_failed"),
                authorization_denials: CountOutcome(report.events, "blocked")
            }

        CASE "pci_token_access":
            // Report all platform token access events
            report.events ← Database.find("audit_logs", {
                timestamp: {$gte: start_date, $lte: end_date},
                event: {$regex: "^platform_token"}
            })

            report.metrics ← {
                token_accesses: CountByEvent(report.events, "platform_token_retrieved"),
                token_refreshes: CountByEvent(report.events, "platform_token_refreshed"),
                token_revocations: CountByEvent(report.events, "platform_token_revoked"),
                decryption_failures: CountByEvent(report.events, "token_decryption_failed")
            }

        CASE "security_incidents":
            // Report all critical security events
            report.events ← Database.find("audit_logs", {
                timestamp: {$gte: start_date, $lte: end_date},
                severity: {$in: ["error", "critical"]}
            })

            report.alerts ← Database.find("security_alerts", {
                triggered_at: {$gte: start_date, $lte: end_date}
            })

            report.metrics ← {
                total_incidents: report.alerts.length,
                critical_incidents: CountBySeverity(report.alerts, "critical"),
                resolved_incidents: CountByStatus(report.alerts, "resolved"),
                false_positives: CountByStatus(report.alerts, "false_positive"),
                average_resolution_time: CalculateAverageResolutionTime(report.alerts)
            }

        DEFAULT:
            RETURN error("Unknown report type: " + report_type)
    END SWITCH

    // Audit the report generation
    RecordAuditEvent(
        event="compliance_report_generated",
        severity="info",
        details={
            report_type: report_type,
            period_days: (end_date - start_date) / 86400,
            event_count: report.events.length,
            generated_by: GetCurrentUserID()
        }
    )

    RETURN report
END
```

**Time Complexity**: O(n) where n = events in date range
**Space Complexity**: O(n)

---

## Algorithm 5: Log Retention and Archival

```
ALGORITHM: ArchiveOldLogs
INPUT: retention_days (integer)
OUTPUT: archived_count (integer)

CONSTANTS:
    BATCH_SIZE = 10000
    ARCHIVE_STORAGE = "s3://audit-logs-archive/"

BEGIN
    cutoff_date ← GetCurrentTimestamp() - (retention_days * 86400)

    // Step 1: Count logs to archive
    total_count ← Database.count("audit_logs", {
        timestamp: {$lt: cutoff_date}
    })

    archived_count ← 0
    offset ← 0

    // Step 2: Process in batches
    WHILE offset < total_count DO
        // Fetch batch
        batch ← Database.find("audit_logs", {
            timestamp: {$lt: cutoff_date}
        }).limit(BATCH_SIZE).skip(offset)

        IF batch is empty THEN
            BREAK
        END IF

        // Step 3: Compress and upload to archive storage
        archive_filename ← "audit_logs_" + FormatDate(cutoff_date) + "_batch_" + (offset / BATCH_SIZE) + ".json.gz"

        compressed_data ← GZIPCompress(JSONStringify(batch))

        S3.upload(
            bucket=ARCHIVE_STORAGE,
            key=archive_filename,
            data=compressed_data,
            metadata={
                record_count: batch.length,
                date_range: batch[0].timestamp + " to " + batch[-1].timestamp
            }
        )

        // Step 4: Delete from primary database
        batch_ids ← batch.map(log => log.id)
        Database.deleteMany("audit_logs", {
            id: {$in: batch_ids}
        })

        archived_count ← archived_count + batch.length
        offset ← offset + BATCH_SIZE

        // Throttle to avoid overloading DB
        Sleep(100) // 100ms
    END WHILE

    // Step 5: Audit the archival
    RecordAuditEvent(
        event="audit_logs_archived",
        severity="info",
        details={
            retention_days: retention_days,
            archived_count: archived_count,
            archive_location: ARCHIVE_STORAGE
        }
    )

    RETURN archived_count
END
```

**Time Complexity**: O(n) where n = logs to archive
**Space Complexity**: O(b) where b = batch size

---

## Critical Security Events

### Authentication Events
```
- authentication_success
- authentication_failed
- authentication_mfa_challenged
- authentication_mfa_failed
- password_reset_requested
- password_changed
- email_verification_sent
```

### Authorization Events
```
- authorization_granted
- authorization_denied_no_permission
- authorization_denied_ownership
- role_assigned
- role_revoked
- permission_escalation_attempt
```

### Token Events
```
- oauth_flow_initiated
- oauth_tokens_issued
- refresh_token_rotated
- refresh_token_reuse_detected (CRITICAL)
- platform_token_stored
- platform_token_refreshed
- platform_token_revoked
- jwt_invalid_signature (CRITICAL)
- token_decryption_failed (CRITICAL)
```

### Rate Limiting Events
```
- rate_limit_exceeded
- burst_detected_warning
- burst_detected_blocked
```

### Security Alerts
```
- brute_force_attempt
- token_theft_suspected
- impossible_travel
- privilege_escalation_attempt
- api_abuse_detected
```

---

## Structured Logging Format (JSON)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-12-06T10:30:45.123456Z",
  "event": "refresh_token_reuse_detected",
  "severity": "critical",
  "user_id": "user_123",
  "ip_address": "203.0.113.45",
  "user_agent": "Mozilla/5.0...",
  "request_id": "req_abc123",
  "resource_type": "refresh_token",
  "resource_id": "token_xyz",
  "action": "rotate",
  "outcome": "blocked",
  "details": {
    "family_id": "family_456",
    "client_id": "client_789",
    "previous_ip": "198.51.100.12",
    "time_since_last_use": 120
  },
  "metadata": {
    "server_id": "web-1",
    "version": "1.2.3",
    "environment": "production"
  },
  "session_id": "session_abc",
  "client_id": "client_789"
}
```

---

## Security Best Practices

### 1. What to Log
- **Authentication events**: All login attempts (success and failure)
- **Authorization events**: All access control decisions
- **Data access**: Reads, writes, deletes of sensitive data
- **Configuration changes**: System settings, role/permission changes
- **Security events**: Token operations, rate limits, anomalies

### 2. What NOT to Log
- **Passwords** (even hashed)
- **Access tokens** (except last 4 characters for debugging)
- **Refresh tokens** (use token ID instead)
- **Encryption keys**
- **Personal data** (PII) unless necessary for audit

### 3. Log Integrity
- **Tamper-proof**: Write-only logs (append-only mode)
- **Immutable**: Never modify or delete logs (archive instead)
- **Signed**: Cryptographically sign critical events
- **Centralized**: Send to separate logging service

### 4. Performance
- **Async logging**: Don't block requests
- **Batching**: Batch writes to reduce I/O
- **Sampling**: Sample debug logs in production (1-10%)
- **Compression**: Compress archived logs (GZIP)

### 5. Retention
- **Hot storage**: 30-90 days in database
- **Cold storage**: 1-7 years in archive (compliance)
- **Critical events**: Never delete (permanent archive)

---

## Complexity Analysis

### Time Complexity
- **RecordAuditEvent**: O(1) amortized with batching
- **QueryAuditLogs**: O(log n + k) where k = results
- **CheckSecurityAlerts**: O(log n) for pattern detection
- **GenerateComplianceReport**: O(n) where n = events in range
- **ArchiveOldLogs**: O(n) where n = logs to archive

### Space Complexity
- **Per Log Entry**: ~500 bytes (JSON)
- **Buffer**: O(b) where b = batch size (100 entries)
- **Archive**: O(n) compressed to ~30% of original size

### Database Indexes
```sql
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id, timestamp DESC);
CREATE INDEX idx_audit_logs_event ON audit_logs(event, timestamp DESC);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity, timestamp DESC);
CREATE INDEX idx_audit_logs_ip ON audit_logs(ip_address, timestamp DESC);
CREATE INDEX idx_audit_logs_request ON audit_logs(request_id);

-- Partial index for critical events
CREATE INDEX idx_audit_logs_critical ON audit_logs(timestamp DESC)
WHERE severity IN ('error', 'critical');
```

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Compliance Standards**: GDPR, SOX, PCI-DSS, HIPAA
**Last Updated**: 2025-12-06
