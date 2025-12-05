---
description: Build a feature based on a plan
argument-hint: [path-to-plan]
allowed-tools: Read, Write, Edit, Bash, Edit, MultiEdit
---

# Build

Follow the `Workflow` to implement the `PATH_TO_PLAN` then `Report` the completed work.

## Variables

PATH_TO_PLAN: $ARGUMENTS

## Workflow

- Read the plan at `PATH_TO_PLAN`. 
- Think hard about the plan and implement it into the codebase.

## Report

- Run `git diff --stat`, then `git diff` to see the total lines changed and the files that changed.
- Report your work in the following yaml format:
<format>
work_changes:
  - file: <file-path>
    lines_changed: <number-of-lines-changed>
    description: <one-sentence-description-of-changes>
  - file: <file-path>
    lines_changed: <number-of-lines-changed>
    description: <one-sentence-description-of-changes>
  - ...
</format>