# ARW CLI Release

Run the full release workflow for the ARW CLI package.

## Usage

```
/arw-release [options]
```

## Options

- `dry-run` - Preview without publishing (default for safety)
- `npm` - Publish to npm only
- `cargo` - Publish to crates.io only
- `full` - Full release to both registries

## Instructions

Run the release workflow based on the requested option:

**Default (dry-run):** Preview release without publishing
```bash
.claude/skills/arw-release-manager/scripts/quick-publish.sh --dry-run
```

**npm:** Publish to npm only
```bash
.claude/skills/arw-release-manager/scripts/quick-publish.sh --npm-only
```

**cargo:** Publish to crates.io only
```bash
.claude/skills/arw-release-manager/scripts/quick-publish.sh --cargo-only
```

**full:** Full release to both registries
```bash
.claude/skills/arw-release-manager/scripts/quick-publish.sh
```

Before running full release, ensure:
1. All tests pass
2. Version has been bumped
3. CHANGELOG has been updated
4. User has confirmed they want to publish

After release, report:
1. Verification results
2. Test results
3. Build status
4. Publication status (npm and/or crates.io)
5. Next steps (git tag, GitHub release)
