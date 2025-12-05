---
allowed-tools: Bash(git ls-files:*), Read
description: Answer questions about the project structure and documentation without coding
---

# Question

Answer the user's question by analyzing the project structure and documentation. This prompt is designed to provide information and answer questions without making any code changes.

## Variables

QUESTION: $ARGUMENTS

## Instructions

- IMPORTANT: Focus on understanding and explaining existing code and project structure
- IMPORTANT: Provide clear, informative answers based on project analysis
- IMPORTANT: If the question requires code changes, explain what would need to be done conceptually without implementing

## Workflow

- `git ls-files` to understand the project structure
- README.md for project overview and documentation

## Response Format

- Direct answer to the question
- Supporting evidence from project structure
- References to relevant documentation
- Conceptual explanations where applicable
