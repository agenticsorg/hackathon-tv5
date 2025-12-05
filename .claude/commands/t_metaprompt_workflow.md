---
allowed-tools: Write, Edit, WebFetch, Task, mcp__firecrawl-mcp__firecrawl_scrape, Fetch
description: Create a new prompt
---

# MetaPrompt

Based on the `High Level Prompt` follow the  `Workflow`, to create a new prompt in the `Specified Format`. Before you start, WebFetch everything in the `Documentation`.

## Variables

HIGH_LEVEL_PROMPT: $ARGUMENTS

## Workflow

- We're building a new prompt to satisfy the request detailed in the `High Level Prompt`.
- Save the new prompt to `.claude/commands/<name_of_prompt>.md`
  - The name of the prompt should make sense based on the `High Level Prompt`
- VERY IMPORTANT: The prompt should be in the `Specified Format`
  - Do not create any additional sections or headers that are not in the `Specified Format`
- IMPORTANT: As you're working through the `Specified Format`, replace every block of `<some request>` with the request detailed within the braces.
- Note we're calling these 'prompts' they're also known as custom slash commands.
- Use one Task tool per documentation item to run sub tasks to gather documentation quickly in parallel using `Task` and `WebFetch`.
- Ultra Think - you're operating a prompt that builds a prompt. Stay focused on the details of creating the best high quality prompt for other ai agents.
- If the `High Level Prompt` requested multiple arguments, give each their own h2 header and then place `$ARGUMENTS` right below their respective h2 header. These prompts use an index based system.
- Note, if no variables are requested or mentioned, do not create a Variables section.
- Think through what the static variables vs dynamic variables are and place them accordingly with dynamic variables coming first and static variables coming second.
  - Prefer the `$1`, `$2`, ... over the `$ARGUMENTS` notation.

## Documentation

Slash Command Documentation: https://docs.anthropic.com/en/docs/claude-code/slash-commands
Create Custom Slash Commands: https://docs.anthropic.com/en/docs/claude-code/common-workflows#create-custom-slash-commands
Available Tools and Settings: https://docs.anthropic.com/en/docs/claude-code/settings

## Specified Format
```md
---
allowed-tools: <allowed-tools comma separated>
description: <description we'll use to id this prompt>
argument-hint: [<argument-hint for the first dynamic variable>], [<argument-hint for the second dynamic variable>]
model: sonnet
---

# <name_of_prompt>

<prompt purpose: here we describe what the prompt does at a high level and reference any sections we create that are relevant like the `Instructions` section. Every prompt must have an `Instructions` section where we detail the instructions for the prompt in a bullet point list>

## Variables

<NAME_OF_DYNAMIC_VARIABLE>: $1
<NAME_OF_DYNAMIC_VARIABLE>: $2
<NAME_OF_STATIC_VARIABLE>: <SOMETHING STATIC>

## Workflow
<step by step numbered list of tasks to complete to accomplish the prompt>

## Report
<details of how the prompt should respond back to the user based on the prompt>

```