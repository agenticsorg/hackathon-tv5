---
allowed-tools: Read, Write, Edit, Glob, Grep, MultiEdit
description: Creates a concise engineering implementation plan based on user requirements and saves it to specs directory
argument-hint: [user prompt] [documentation urls] [relevant files]
model: claude-sonnet-4-5-20250929
---

# Quick Plan

Create a detailed implementation plan based on the user's requirements provided through the `USER_PROMPT` variable. Analyze the request, pull in the documentation, think through the implementation approach, and save a comprehensive specification document to `PLAN_OUTPUT_DIRECTORY/<name-of-plan>.md` that can be used as a blueprint for actual development work. Follow the `Instructions` and work through the `Workflow` to create the plan.

## Variables

USER_PROMPT: $1
DOCUMENTATION_URLS: $2
RELEVANT_FILES_COLLECTION: $3
PLAN_OUTPUT_DIRECTORY: `specs/`
DOCUMENTATION_OUTPUT_DIRECTORY: `ai_docs/`

## Instructions

- IMPORTANT: If no `USER_PROMPT`, `DOCUMENTATION_URLS`, or `RELEVANT_FILES_COLLECTION` is provided, stop and ask the user to provide them.
- READ the `RELEVANT_FILES_COLLECTION` file which contains a structured bullet point list of files with line ranges (format: `- <path to file> (offset: N, limit: M)`)
- Carefully analyze the user's requirements provided in the USER_PROMPT variable
- With Task, in parallel, scrap each DOCUMENTATION_URLS with firecrawl (or webfetch if firecrawl is not available)
  - Instruct the subagent to save each piece of documentation to `DOCUMENTATION_OUTPUT_DIRECTORY/<name-of-documentation>.md`
  - Instruct the subagent to return the path to each piece of documentation for future reference
  - Then echo the path to each piece of documentation for future reference
- Think deeply (ultrathink) about the best approach to implement the requested functionality or solve the problem
- READ the files listed in the `RELEVANT_FILES_COLLECTION` using the specified offset and limit values to help you understand the codebase and implement the plan.
  - Use these as a starting point, do not limit yourself to these files.
  - Double check the codebase for any other files that may be relevant to planning the task to fulfill the `USER_PROMPT`.
  - Use these to build up a `## Relevant Files` section in your plan that you will instruct the builder agents to read before implementing the plan.
- Create a concise implementation plan that includes:
  - Clear problem statement and objectives
  - Technical approach and architecture decisions
  - Step-by-step implementation guide
  - Potential challenges and solutions
  - Testing strategy
  - Success criteria
- Generate a descriptive, kebab-case filename based on the main topic of the plan
- Be sure to include a '## Validation Command' step for the builder agents that will run your plan. This section proves that the work is complete.
- Be sure to include a '## Relevant Files' step for the builder agents that will run your plan. This section lists the files that are relevant to the plan.
  - IMPORTANT: Include references to the generated documentation in the '## Relevant Files' section and instruct the builder agents to read the documentation.
- Save the complete implementation plan to `PLAN_OUTPUT_DIRECTORY/<descriptive-name>.md`
- Ensure the plan is detailed enough that another developer could follow it to implement the solution
- Include code examples or pseudo-code where appropriate to clarify complex concepts
- Consider edge cases, error handling, and scalability concerns
- Structure the document with clear sections and proper markdown formatting

## Workflow

1. Analyze Requirements - THINK HARD and parse the USER_PROMPT to understand the core problem and desired outcome
2. Scrap Documentation - With Task, in parallel, scrap each DOCUMENTATION_URLS with firecrawl (or webfetch if firecrawl is not available)
3. Design Solution - Develop technical approach including architecture decisions and implementation strategy
4. Document Plan - Structure a comprehensive markdown document with problem statement, implementation steps, and testing approach
5. Generate Filename - Create a descriptive kebab-case filename based on the plan's main topic
6. Save & Report - Follow the `Report` section to write the plan to `PLAN_OUTPUT_DIRECTORY/<filename>.md` and provide a summary of key components

## Report

After creating and saving the implementation plan, provide a concise report with the following format:

```
✅ Implementation Plan Created

File: PLAN_OUTPUT_DIRECTORY/<filename>.md
Topic: <brief description of what the plan covers>
Key Components:
- <main component 1>
- <main component 2>
- <main component 3>
```