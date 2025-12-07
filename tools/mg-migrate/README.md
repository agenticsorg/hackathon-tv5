# mg-migrate

Database migration CLI tool for Media Gateway platform.

## Installation

Build from source:

```bash
cargo build --package mg-migrate --release
```

The binary will be available at `target/release/mg-migrate`.

## Configuration

Set the `DATABASE_URL` environment variable or use the `--database-url` flag:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/media_gateway"
```

## Commands

### Apply Migrations

Apply all pending migrations:

```bash
mg-migrate up
```

Preview migrations without applying (dry-run):

```bash
mg-migrate up --dry-run
```

### Rollback Migrations

Rollback the last migration:

```bash
mg-migrate down
```

Rollback multiple migrations:

```bash
mg-migrate down --steps 3
```

Preview rollback without applying:

```bash
mg-migrate down --dry-run
```

### Migration Status

Show status of all migrations:

```bash
mg-migrate status
```

Output example:

```
Migration Status

Version              Name                                     Status
────────────────────────────────────────────────────────────────────────────────
010                  create_users                             APPLIED
011                  add_user_preferences                     APPLIED
012                  create_audit_logs                        APPLIED
013                  add_quality_score                        PENDING
014                  add_parental_controls                    PENDING

Summary: 5 total, 3 applied, 2 pending
```

### Create New Migration

Create a new migration file:

```bash
mg-migrate create add_new_feature
```

This creates a new file in `migrations/` with the next version number:

```
migrations/018_add_new_feature.sql
```

## Migration File Format

Migration files should be named with the format: `{version}_{name}.sql`

- `version`: Zero-padded 3-digit number (e.g., `001`, `010`, `123`)
- `name`: Descriptive snake_case name

Example:

```sql
-- Migration: create_users
-- Description: Create users table with authentication

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

## Schema Tracking

The tool creates a `schema_migrations` table to track applied migrations:

```sql
CREATE TABLE schema_migrations (
    version VARCHAR(255) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Exit Codes

- `0`: Success
- `1`: Error occurred

## CI/CD Integration

Use in CI/CD pipelines:

```bash
# GitHub Actions example
- name: Run migrations
  run: |
    export DATABASE_URL="${{ secrets.DATABASE_URL }}"
    ./target/release/mg-migrate up
```

## Features

- Transaction-based migrations (atomic apply/rollback)
- Dry-run mode for previewing changes
- Color-coded output for status
- Version tracking in database
- Support for PostgreSQL
- CI/CD friendly exit codes

## Examples

### Local Development

```bash
# Check migration status
mg-migrate status

# Apply pending migrations
mg-migrate up

# Create new migration
mg-migrate create add_email_verification
```

### Production Deployment

```bash
# Preview changes first
mg-migrate up --dry-run

# Apply migrations
mg-migrate up

# Verify status
mg-migrate status
```

### Rollback

```bash
# Preview rollback
mg-migrate down --dry-run

# Rollback last migration
mg-migrate down

# Rollback last 2 migrations
mg-migrate down --steps 2
```

## Notes

- All migrations run within transactions for atomicity
- Rollback only removes the migration record from `schema_migrations`
- Manual cleanup may be required for rollbacks (no down migrations executed)
- Migration files must exist in the `migrations/` directory relative to where the tool is run
