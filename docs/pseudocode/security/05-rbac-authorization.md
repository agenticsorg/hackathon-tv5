# RBAC Authorization System - Pseudocode Specification

## Overview
Role-Based Access Control (RBAC) provides fine-grained authorization using roles, permissions, and resource ownership. This implementation supports hierarchical roles, dynamic permissions, and resource-level access control.

---

## Data Structures

```
STRUCTURE Permission:
    id: UUID
    name: String (e.g., "users:read", "videos:delete")
    resource: String (e.g., "users", "videos", "playlists")
    action: Enum["create", "read", "update", "delete", "manage"]
    description: String
    requires_ownership: Boolean (default: false)
    conditions: Array<Condition> (optional, for dynamic permissions)

STRUCTURE Role:
    id: UUID
    name: String (e.g., "admin", "moderator", "user")
    display_name: String
    description: String
    permissions: Array<Permission>
    inherits_from: Array<Role> (role hierarchy)
    priority: Integer (higher = more privileged)
    system_role: Boolean (cannot be deleted)

STRUCTURE UserRole:
    user_id: String
    role_id: UUID
    granted_at: Timestamp
    granted_by: String (user_id of admin)
    expires_at: Timestamp (null = permanent)
    scope: String (optional, for scoped roles like "org:123")

STRUCTURE ResourceOwnership:
    resource_type: String (e.g., "video", "playlist")
    resource_id: String
    owner_id: String (user_id)
    created_at: Timestamp
    additional_owners: Array<String> (for shared ownership)

STRUCTURE Condition:
    type: Enum["time_based", "ip_based", "attribute_based"]
    parameters: Object
    // Example: {type: "time_based", parameters: {start: "09:00", end: "17:00"}}
```

---

## Algorithm 1: Check Authorization

```
ALGORITHM: CheckAuthorization
INPUT: user_id (string), action (string), resource_type (string), resource_id (string, optional)
OUTPUT: authorized (boolean) and reason (string)

BEGIN
    // Step 1: Get user roles (with caching)
    cache_key ← "user_roles:" + user_id
    user_roles ← Cache.get(cache_key)

    IF user_roles is null THEN
        user_roles ← Database.find("user_roles", {
            user_id: user_id,
            $or: [
                {expires_at: null},
                {expires_at: {$gt: GetCurrentTimestamp()}}
            ]
        })

        // Cache for 5 minutes
        Cache.setWithTTL(cache_key, user_roles, 300)
    END IF

    IF user_roles is empty THEN
        AuditLog.record(
            event="authorization_no_roles",
            user_id=user_id,
            action=action,
            resource_type=resource_type,
            severity="info"
        )
        RETURN {authorized: false, reason: "User has no assigned roles"}
    END IF

    // Step 2: Expand roles with inheritance
    expanded_roles ← ExpandRoleHierarchy(user_roles)

    // Step 3: Collect all permissions from roles
    all_permissions ← []
    FOR EACH role IN expanded_roles DO
        role_data ← GetRoleByID(role.role_id) // Cached lookup

        FOR EACH permission IN role_data.permissions DO
            all_permissions.append(permission)
        END FOR
    END FOR

    // Step 4: Find matching permissions
    permission_name ← resource_type + ":" + action

    matching_permissions ← []
    FOR EACH permission IN all_permissions DO
        IF permission.name == permission_name OR permission.name == resource_type + ":manage" THEN
            matching_permissions.append(permission)
        END IF
    END FOR

    IF matching_permissions is empty THEN
        AuditLog.record(
            event="authorization_denied_no_permission",
            user_id=user_id,
            action=action,
            resource_type=resource_type,
            resource_id=resource_id,
            severity="info"
        )
        RETURN {authorized: false, reason: "No permission for action: " + permission_name}
    END IF

    // Step 5: Check ownership if required
    FOR EACH permission IN matching_permissions DO
        // Check dynamic conditions
        IF permission.conditions is not empty THEN
            IF NOT EvaluateConditions(permission.conditions, user_id) THEN
                CONTINUE // Skip this permission
            END IF
        END IF

        // Check ownership requirement
        IF permission.requires_ownership is true AND resource_id is not null THEN
            is_owner ← CheckResourceOwnership(user_id, resource_type, resource_id)

            IF NOT is_owner THEN
                CONTINUE // Check next permission
            END IF
        END IF

        // Permission granted
        AuditLog.record(
            event="authorization_granted",
            user_id=user_id,
            action=action,
            resource_type=resource_type,
            resource_id=resource_id,
            permission=permission.name,
            severity="debug"
        )

        RETURN {authorized: true, reason: "Permission: " + permission.name}
    END FOR

    // Step 6: All permissions failed (likely ownership check)
    AuditLog.record(
        event="authorization_denied_ownership",
        user_id=user_id,
        action=action,
        resource_type=resource_type,
        resource_id=resource_id,
        severity="info"
    )

    RETURN {authorized: false, reason: "Permission requires resource ownership"}
END
```

**Time Complexity**: O(r × p) where r = number of roles, p = permissions per role
**Space Complexity**: O(r × p) for permission collection

---

## Algorithm 2: Expand Role Hierarchy

```
ALGORITHM: ExpandRoleHierarchy
INPUT: user_roles (array of UserRole)
OUTPUT: expanded_roles (array of Role with inheritance)

BEGIN
    expanded_roles ← []
    visited_roles ← SET() // Prevent circular inheritance

    FOR EACH user_role IN user_roles DO
        ExpandRoleRecursive(user_role.role_id, expanded_roles, visited_roles)
    END FOR

    // Sort by priority (highest first)
    expanded_roles.sortByDescending(priority)

    RETURN expanded_roles
END

SUBROUTINE: ExpandRoleRecursive
INPUT: role_id (UUID), expanded_roles (array), visited_roles (set)
OUTPUT: none (modifies expanded_roles)

BEGIN
    // Prevent infinite recursion
    IF role_id IN visited_roles THEN
        RETURN
    END IF

    visited_roles.add(role_id)

    // Get role data
    role ← GetRoleByID(role_id)

    IF role is null THEN
        RETURN
    END IF

    // Add current role
    expanded_roles.append(role)

    // Recursively expand inherited roles
    FOR EACH parent_role_id IN role.inherits_from DO
        ExpandRoleRecursive(parent_role_id, expanded_roles, visited_roles)
    END FOR
END
```

**Time Complexity**: O(r × d) where r = number of roles, d = inheritance depth
**Space Complexity**: O(r)

---

## Algorithm 3: Check Resource Ownership

```
ALGORITHM: CheckResourceOwnership
INPUT: user_id (string), resource_type (string), resource_id (string)
OUTPUT: is_owner (boolean)

BEGIN
    // Step 1: Check cache
    cache_key ← "ownership:" + resource_type + ":" + resource_id + ":" + user_id
    cached_result ← Cache.get(cache_key)

    IF cached_result is not null THEN
        RETURN cached_result
    END IF

    // Step 2: Query ownership
    ownership ← Database.findOne("resource_ownership", {
        resource_type: resource_type,
        resource_id: resource_id
    })

    IF ownership is null THEN
        // Resource not found - deny access
        Cache.setWithTTL(cache_key, false, 60)
        RETURN false
    END IF

    // Step 3: Check primary owner
    IF ownership.owner_id == user_id THEN
        Cache.setWithTTL(cache_key, true, 300)
        RETURN true
    END IF

    // Step 4: Check additional owners
    IF user_id IN ownership.additional_owners THEN
        Cache.setWithTTL(cache_key, true, 300)
        RETURN true
    END IF

    // Step 5: Not an owner
    Cache.setWithTTL(cache_key, false, 60)
    RETURN false
END
```

**Time Complexity**: O(1) with indexed database lookup
**Space Complexity**: O(1)

---

## Algorithm 4: Evaluate Dynamic Conditions

```
ALGORITHM: EvaluateConditions
INPUT: conditions (array of Condition), user_id (string)
OUTPUT: all_conditions_met (boolean)

BEGIN
    FOR EACH condition IN conditions DO
        result ← EvaluateSingleCondition(condition, user_id)

        IF NOT result THEN
            RETURN false // All conditions must be met
        END IF
    END FOR

    RETURN true
END

SUBROUTINE: EvaluateSingleCondition
INPUT: condition (Condition), user_id (string)
OUTPUT: condition_met (boolean)

BEGIN
    SWITCH condition.type:
        CASE "time_based":
            current_time ← GetCurrentTime()
            start_time ← ParseTime(condition.parameters.start)
            end_time ← ParseTime(condition.parameters.end)

            IF current_time >= start_time AND current_time <= end_time THEN
                RETURN true
            ELSE
                RETURN false
            END IF

        CASE "ip_based":
            user_ip ← GetUserIPAddress(user_id)
            allowed_ips ← condition.parameters.allowed_ips
            allowed_cidrs ← condition.parameters.allowed_cidrs

            // Check exact IP match
            IF user_ip IN allowed_ips THEN
                RETURN true
            END IF

            // Check CIDR range match
            FOR EACH cidr IN allowed_cidrs DO
                IF IPInCIDR(user_ip, cidr) THEN
                    RETURN true
                END IF
            END FOR

            RETURN false

        CASE "attribute_based":
            user ← GetUserByID(user_id)
            attribute_name ← condition.parameters.attribute
            required_value ← condition.parameters.value
            operator ← condition.parameters.operator // "equals", "contains", "gt", "lt"

            user_value ← user[attribute_name]

            SWITCH operator:
                CASE "equals":
                    RETURN user_value == required_value
                CASE "contains":
                    RETURN required_value IN user_value
                CASE "gt":
                    RETURN user_value > required_value
                CASE "lt":
                    RETURN user_value < required_value
                DEFAULT:
                    RETURN false
            END SWITCH

        DEFAULT:
            AuditLog.record(
                event="unknown_condition_type",
                condition_type=condition.type,
                severity="warning"
            )
            RETURN false
    END SWITCH
END
```

**Time Complexity**: O(c) where c = number of conditions
**Space Complexity**: O(1)

---

## Algorithm 5: Assign Role to User

```
ALGORITHM: AssignRoleToUser
INPUT: user_id (string), role_id (UUID), granted_by (string), expires_at (timestamp, optional)
OUTPUT: success or error

BEGIN
    // Step 1: Validate role exists
    role ← GetRoleByID(role_id)

    IF role is null THEN
        RETURN error("Role not found")
    END IF

    // Step 2: Check if admin has permission to grant this role
    admin_authorized ← CheckAuthorization(
        user_id=granted_by,
        action="manage",
        resource_type="roles",
        resource_id=role_id
    )

    IF NOT admin_authorized.authorized THEN
        AuditLog.record(
            event="unauthorized_role_assignment",
            admin_id=granted_by,
            target_user_id=user_id,
            role_id=role_id,
            severity="warning"
        )
        RETURN error("Unauthorized to assign this role")
    END IF

    // Step 3: Check if user already has this role
    existing_assignment ← Database.findOne("user_roles", {
        user_id: user_id,
        role_id: role_id,
        $or: [
            {expires_at: null},
            {expires_at: {$gt: GetCurrentTimestamp()}}
        ]
    })

    IF existing_assignment is not null THEN
        RETURN error("User already has this role")
    END IF

    // Step 4: Create role assignment
    Database.insert("user_roles", {
        user_id: user_id,
        role_id: role_id,
        granted_at: GetCurrentTimestamp(),
        granted_by: granted_by,
        expires_at: expires_at,
        scope: null
    })

    // Step 5: Invalidate user role cache
    Cache.delete("user_roles:" + user_id)

    // Step 6: Audit log
    AuditLog.record(
        event="role_assigned",
        user_id=user_id,
        role_id=role_id,
        role_name=role.name,
        granted_by=granted_by,
        expires_at=expires_at,
        severity="info"
    )

    RETURN success
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Algorithm 6: Create Custom Role

```
ALGORITHM: CreateCustomRole
INPUT: role_data (Role), created_by (string)
OUTPUT: role_id (UUID) or error

BEGIN
    // Step 1: Validate creator has permission
    authorized ← CheckAuthorization(
        user_id=created_by,
        action="create",
        resource_type="roles",
        resource_id=null
    )

    IF NOT authorized.authorized THEN
        RETURN error("Unauthorized to create roles")
    END IF

    // Step 2: Validate role name is unique
    existing_role ← Database.findOne("roles", {
        name: role_data.name
    })

    IF existing_role is not null THEN
        RETURN error("Role name already exists")
    END IF

    // Step 3: Validate permissions exist
    FOR EACH permission IN role_data.permissions DO
        perm ← Database.findOne("permissions", {
            id: permission.id
        })

        IF perm is null THEN
            RETURN error("Invalid permission ID: " + permission.id)
        END IF
    END FOR

    // Step 4: Validate inheritance (prevent circular references)
    FOR EACH parent_role_id IN role_data.inherits_from DO
        IF WouldCreateCircularInheritance(role_data.id, parent_role_id) THEN
            RETURN error("Circular role inheritance detected")
        END IF
    END FOR

    // Step 5: Create role
    role_id ← GenerateUUID()

    Database.insert("roles", {
        id: role_id,
        name: role_data.name,
        display_name: role_data.display_name,
        description: role_data.description,
        permissions: role_data.permissions,
        inherits_from: role_data.inherits_from,
        priority: role_data.priority,
        system_role: false,
        created_by: created_by,
        created_at: GetCurrentTimestamp()
    })

    // Step 6: Invalidate role cache
    Cache.delete("roles")

    // Step 7: Audit log
    AuditLog.record(
        event="role_created",
        role_id=role_id,
        role_name=role_data.name,
        created_by=created_by,
        severity="info"
    )

    RETURN role_id
END
```

**Time Complexity**: O(p) where p = number of permissions to validate
**Space Complexity**: O(1)

---

## Predefined System Roles

```
// Administrator Role
ROLE: admin
PERMISSIONS:
    - *:manage (all resources, all actions)
PRIORITY: 1000
SYSTEM_ROLE: true

// Moderator Role
ROLE: moderator
PERMISSIONS:
    - users:read
    - users:update
    - videos:read
    - videos:update
    - videos:delete
    - comments:manage
    - reports:manage
PRIORITY: 500
INHERITS_FROM: []
SYSTEM_ROLE: true

// Content Creator Role
ROLE: creator
PERMISSIONS:
    - videos:create
    - videos:read (ownership required)
    - videos:update (ownership required)
    - videos:delete (ownership required)
    - playlists:create
    - playlists:manage (ownership required)
PRIORITY: 100
INHERITS_FROM: [user]
SYSTEM_ROLE: true

// Standard User Role
ROLE: user
PERMISSIONS:
    - videos:read
    - playlists:read
    - comments:create
    - comments:update (ownership required)
    - comments:delete (ownership required)
PRIORITY: 10
INHERITS_FROM: []
SYSTEM_ROLE: true
```

---

## Security Best Practices

### 1. Principle of Least Privilege
- Grant minimum permissions required
- Use ownership-based permissions for user-generated content
- Regularly audit role assignments

### 2. Role Hierarchy
- Design clear role hierarchy (admin > moderator > creator > user)
- Use inheritance to reduce permission duplication
- Prevent circular inheritance

### 3. Permission Naming Convention
```
<resource>:<action>
Examples:
- videos:read
- users:delete
- playlists:manage (implies all actions)
```

### 4. Caching Strategy
- Cache user roles for 5 minutes
- Cache role definitions indefinitely (invalidate on update)
- Cache ownership checks for 5 minutes (short TTL for dynamic data)

### 5. Audit Events
- `authorization_granted` - Access allowed
- `authorization_denied_no_permission` - Missing permission
- `authorization_denied_ownership` - Not resource owner
- `role_assigned` - Role granted to user
- `role_revoked` - Role removed from user
- `role_created` - New role created
- `unauthorized_role_assignment` - Attempted unauthorized assignment

---

## Complexity Analysis

### Time Complexity
- **CheckAuthorization**: O(r × p) where r = roles, p = permissions
- **ExpandRoleHierarchy**: O(r × d) where d = inheritance depth
- **CheckResourceOwnership**: O(1) with indexing
- **AssignRoleToUser**: O(1)
- **CreateCustomRole**: O(p) for permission validation

### Space Complexity
- **Role Storage**: O(r) where r = number of roles
- **Permission Storage**: O(p) where p = number of permissions
- **User Role Assignments**: O(u × r) where u = users

### Database Indexes
```sql
-- User roles
CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_expires ON user_roles(expires_at);

-- Resource ownership
CREATE INDEX idx_ownership_resource ON resource_ownership(resource_type, resource_id);
CREATE INDEX idx_ownership_owner ON resource_ownership(owner_id);

-- Roles
CREATE UNIQUE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_roles_priority ON roles(priority);
```

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Authorization Model**: RBAC with hierarchical roles and ownership
**Last Updated**: 2025-12-06
