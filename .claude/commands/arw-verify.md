# ARW CLI Verify

Run pre-release verification checks for the ARW CLI package.

## Usage

```
/arw-verify
```

## Instructions

Run the verification script to check if the package is ready for release:

```bash
.claude/skills/arw-release-manager/scripts/verify-package.sh
```

This checks:
- Git status (clean working tree)
- package.json validity and required fields
- Cargo.toml validity and required fields
- Semantic version format
- Version synchronization between npm and cargo
- Built files exist
- CLI executables have shebangs
- No hardcoded secrets or API keys
- Documentation files present (README, LICENSE, CHANGELOG)
- Package size is reasonable

After verification, report:
1. Overall status (ready/not ready)
2. Any errors that must be fixed
3. Any warnings to consider
4. Recommendations for next steps
