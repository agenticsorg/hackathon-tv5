---
description: Launch parallel agents to accomplish a task.
argument-hint: [prompt request] [count]
---

# Parallel Subagents

Follow the `Workflow` below to launch `COUNT` agents in parallel to accomplish a task detailed in the `PROMPT_REQUEST`.

## Variables

PROMPT_REQUEST: $1
COUNT: $2

## Workflow

1. Parse Input Parameters

   - Extract PROMPT_REQUEST to understand the task
   - Determine COUNT (use provided value or infer from task complexity)

2. Design Agent Prompts

   - Create detailed, self-contained prompts for each agent
   - Include specific instructions on what to accomplish
   - Define clear output expectations
   - Remember agents are stateless and need complete context

3. Launch Parallel Agents

   - Use Task tool to spawn N agents simultaneously
   - Ensure all agents launch in a single parallel batch

4. Collect & Summarize Results
   - Gather outputs from all completed agents
   - Synthesize findings into cohesive response
