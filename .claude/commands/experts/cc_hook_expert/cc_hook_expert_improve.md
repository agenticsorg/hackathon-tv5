---
description: Review hook changes and update expert knowledge with improvements
---

# Claude Code Hook Expert - Improve

You are a Claude Code Hook Expert specializing in continuous improvement. You will analyze recent hook-related changes, identify patterns and best practices, and update the plan and build expert commands with new learnings to maintain cutting-edge expertise.

## Variables

None required - this command analyzes recent work automatically

## Instructions

- Review all recent changes to hook-related files
- Identify successful patterns and potential improvements
- Extract learnings from implementation experiences
- Update ONLY the ## Expertise sections of expert commands with new knowledge
- Document discovered best practices
- Ensure expert knowledge stays current while keeping workflows stable

## Workflow

1. **Establish Expertise**

   - Read research/uv-scripts-guide.md
   - Read research/claude-code-hooks.md
   - Read research/claude-code-slash-commands.md

2. **Analyze Recent Changes**

   - Run `git diff` to examine uncommitted changes
   - Run `git diff --cached` for staged changes
   - Run `git log --oneline -10` to review recent commits
   - Focus on hook-related files:
     - `.claude/hooks/*.py` - Hook implementations
     - `.claude/settings*.json` - Hook configurations
     - `specs/experts/cc_hook_expert/*.md` - Hook specifications

3. **Determine Relevance**
   Evaluate if changes contain new expertise worth capturing:

   - New hook patterns or techniques discovered?
   - Better error handling or security measures found?
   - Performance optimizations or testing approaches improved?
   - UV script dependencies or configurations refined?
   - Has a new file been added or deleted? Does it warrant an update to the expertise?

   IMPORTANT: **If no relevant learnings found → STOP HERE and report "No expertise updates needed"**

4. **Extract and Apply Learnings**
   If relevant changes found, determine which expert needs updating:

   **For Planning Knowledge** (update cc_hook_expert_plan.md ## Expertise):

   - New event usage patterns
   - Specification structure improvements
   - Security planning considerations
   - Output format decision criteria

   **For Building Knowledge** (update cc_hook_expert_build.md ## Expertise):

   - Implementation patterns and standards
   - UV script configurations
   - Error handling techniques
   - Testing approaches

   Update ONLY the ## Expertise sections with discovered knowledge.
   Do NOT modify Workflow sections - they remain stable.

5. **Report**
   - Follow the `Report` section to report your work.

## Report

Provide improvement summary:

1. **Changes Analyzed**

   - Files reviewed via git diff
   - Hook-related changes identified
   - Relevance determination

2. **Learnings Extracted**

   - New patterns discovered (or "No relevant learnings found")
   - Knowledge worth capturing
   - Improvements identified

3. **Expert Updates Made**
   - Updates to cc_hook_expert_plan.md ## Expertise (if any)
   - Updates to cc_hook_expert_build.md ## Expertise (if any)
   - Or report: "No expertise updates needed - current knowledge remains current"
