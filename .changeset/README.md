# Changesets

This directory contains changesets for the ARW specification.

## Usage

To create a new changeset:

```bash
npm run changeset:add
```

This will prompt you to describe the changes and select the appropriate version bump (major, minor, or patch).

## Versioning the Specification

When you're ready to version the specification:

```bash
npm run version
```

This will:

1. Update the version in `package.json`
2. Update the changelog
3. Remove consumed changesets

## Semantic Versioning for Specifications

For the ARW specification, we follow semantic versioning:

- **Major (1.0.0 → 2.0.0)**: Breaking changes to the specification that would require existing implementations to be updated
- **Minor (1.0.0 → 1.1.0)**: New features or additions to the specification that are backwards compatible
- **Patch (1.0.0 → 1.0.1)**: Clarifications, typo fixes, or non-breaking improvements to the documentation
