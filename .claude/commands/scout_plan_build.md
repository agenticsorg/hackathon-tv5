---
description: Run a three step engineering workflow to deliver on the `USER_PROMPT`
argument-hint: [user-prompt] [documentation-urls]
model: claude-sonnet-4-5-20250929
---

# Scout Plan Build

# Purpose

Run a three step engineering workflow to deliver on the `USER_PROMPT`.

First we scout the codebase for files needed to complete the task.
Then we plan the task based on the files found.
Then we build the task based on the plan.

## Variables

USER_PROMPT: $1
DOCUMENTATION_URLS: $2

## Instructions

- We're executing a three step engineering workflow to deliver on the `USER_PROMPT`.
- Execute each step in order, top to bottom.
- If you're returned an unexpected result, stop and notify the user.
- Place each argument for the SlashCommands arguments within double quotes and convert any nested double quotes to single quotes.
- Do not alter the `USER_PROMPT` variable in anyway, pass it in as is.
- IMPORTANT: Flow through each step in the workflow in order, top to bottom. Only waiting for the previous step to complete before starting the next step. Do not stop in between steps. Complete every step in the workflow before stopping.

## Workflow
> Run the workflow in order, top to bottom. Do not stop in between steps. Complete every step in the workflow before stopping.

1. Run SlashCommand(`/scout "[USER_PROMPT]" "4"`) -> `relevant_files_collection_path`
2. Run SlashCommand(`/plan_w_docs "[USER_PROMPT]" "[DOCUMENTATION_URLS]" "[relevant_files_collection_path]"`) -> `path_to_plan`
3. Run SlashCommand(`/build "[path_to_plan]"`) -> `build_report`
4. Finally, report the work done based on the `Report` section.

## Report

- Deal the success or failure of each step of the workflow to the user.
- Give a high level summary of the work done based on the `build_report`.
