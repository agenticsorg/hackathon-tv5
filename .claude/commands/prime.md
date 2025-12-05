---
description: Gain a general understanding of the codebase
---

# Prime

Execute the `Workflow` and `Report` sections to understand the codebase then summarize your understanding.

## Repository Overview

This is the **Agent-Ready Web (ARW) Specification Repository** - an infrastructure specification and tooling project that includes:

- The ARW specification defining an open standard for efficient agent-web interaction
- A production marketing website deployed to Vercel
- Development tools (Rust CLI, validators, inspector, DevTools extension)
- Reference implementation examples

**Core Philosophy:** ARW is infrastructure for the agent-ready web. Technically enforce actions (OAuth), provide efficiency through machine views (85% token reduction), and enable observability via AI-\* headers.

## Instructions

- This repository contains both specification work AND implementation code
- When working on specification content, maintain consistency with the CloudCart example used throughout
- The marketing website (`www/`) is a production React Router v7 app deployed to Vercel
- All documentation should follow the established patterns and terminology
- When making changes, ensure they align with the core philosophy: infrastructure layer for agent-web interaction, technical enforcement for actions (OAuth), advisory but observable policies for content, efficiency (85% token reduction) as primary value prop
- Format all changes with `npm run format` before committing (from root)
- For website changes, test locally with `cd www && npm run dev` before deploying

## Codebase Structure

- `spec/` - ARW technical specifications
  - `ARW-0.1-draft.md` - **Editor's draft specification** (Work in Progress)
- `schemas/` - JSON schemas and LinkML models
  - `arw_model.yaml` - LinkML schema (authoritative)
  - `arw_model.json` - JSON Schema for llms.txt
  - `arw-manifest.schema.json` - Schema for .well-known/arw-manifest.json
  - `arw-content-index.schema.json` - Schema for .well-known/arw-content-index.json
  - `arw-policies.schema.json` - Schema for .well-known/arw-policies.json
- `docs/` - Organized documentation
  - `arw-overview/` - ARW specification overview and comparisons
  - `discovery/` - Discovery mechanism guides
  - `github/` - GitHub Actions setup guides
  - `vercel/` - Vercel deployment troubleshooting
  - `www/` - Website-specific documentation
  - `arw-inspector/` - Inspector tool documentation
  - `business model/` - Business and stakeholder analysis
- `www/` - Production marketing website (React Router v7 + Tailwind CSS)
  - **Deployed to Vercel** (main = production, preview = staging)
  - Demonstrates ARW concepts and features
  - Includes working `/llms.txt`, `/.llm.md`, and `.well-known/` files
  - GitHub URLs configured in `www/app/config/github.ts`
  - SSR enabled with Vercel serverless functions
- `packages/` - Core tooling packages
  - `cli/` - Rust CLI tool for ARW implementation
    - Generate machine views, validate ARW compliance, dev server
    - Commands: `generate`, `validate`, `init`, `serve`, `scan`, `policy`
    - Comprehensive documentation in `packages/cli/README.md`
  - `validators/` - Python/Node.js validators for ARW manifests
    - `validate-arw.py` - Python validator with jsonschema
    - `validate-arw.mjs` - Node.js validator with ajv
  - `schemas/` - LinkML + JSON schemas for validation
  - `badges/` - Compliance level badges (SVG)
- `devtools/` - Developer tools
  - `arw-devtools-extension/` - Chrome DevTools extension (stub)
    - Native DevTools integration for ARW inspection
    - No CORS issues, works on localhost
- `platform/apps/` - Web applications
  - `arw-inspector/` - Visual ARW inspector web app (React)
    - Browser-based interface for inspecting ARW implementations
    - Side-by-side machine view comparison, chunk identification
- `badges/` - ARW compliance level badges (SVG)
  - Discovery-ready, Semantic-ready, Action-ready, Protocol-ready
- `examples/` - Reference implementations
  - `basic-blog/` - Simple blog demonstrating ARW content discovery
  - `arw-acp-prototype/` - Full ARW + ACP integration with e-commerce
  - `api-docs/` - API documentation example
- `.github/` - GitHub configuration
  - `workflows/` - GitHub Actions for automated Vercel deployments and validation
  - `VERCEL_SETUP.md` - Complete GitHub Actions setup guide
- `.claude/` - Claude Code configuration
  - `commands/` - Custom slash commands for workflow automation
  - `agents/` - Sub-agent definitions for specialized tasks
  - `hooks/` - Python hooks for security, logging, and context tracking
  - `skills/` - UX designer skill for interface work
- `research/` - Fetched documentation and research materials
- `plans/` - Engineering plans and specifications
- `specs/` - Additional specification documents
- `memory/` - Session logs and context bundles (gitignored)
- `.changeset/` - Semantic versioning configuration for the specification

## Key Concepts to Understand

1. **Machine Views** - `.llm.md` files providing Markdown representations optimized for LLM parsing
2. **Discovery Files**:
   - `llms.txt` - YAML manifest (human-readable table of contents)
   - `/.well-known/arw-manifest.json` - Machine-optimized JSON entrypoint
   - `/.well-known/arw-content-index.json` - Paginated content index
   - `/.well-known/arw-policies.json` - Machine-readable policies
   - `sitemap.xml` - Standard web sitemap for dates
   - `robots.txt` - Crawl rules + ARW discovery hints
3. **Declarative Actions** - OAuth-enforced JSON-LD endpoints for transactions (technically protected)
4. **Content Policies** - Machine-readable usage terms (advisory like robots.txt, but provide legal foundation)
5. **Observability** - `AI-*` headers to identify and track AI agent traffic
6. **Protocol Interoperability** - How ARW complements MCP, ACP, and A2A protocols
7. **CloudCart Example** - The consistent fictitious e-commerce platform used throughout all examples
8. **Schema Validation** - LinkML and JSON schemas for manifest validation
9. **Deployment Architecture** - Multi-environment setup with Vercel and GitHub Actions
10. **Package Management** - Root uses pnpm (spec dependencies), www/ uses npm (website dependencies)
11. **Branch Strategy** - `main` for production, `preview` for staging, feature branches for PRs
12. **Tools Directory** - Consolidated directory (`tools/`) containing all development tools
13. **CLI Tools** - Rust CLI (`tools/npx-arw/`) and Node.js validator (`tools/arw-validator/`)
14. **Developer Tools** - Inspector web app and Chrome DevTools extension in `tools/`
15. **Compliance Badges** - Four-level badge system (Discovery, Semantic, Action, Protocol ready)
16. **Validation Pipeline** - Python and Node.js validators, GitHub Actions CI/CD

## Workflow

- Run `git ls-files` to list all files in the repository
- Read these core files:
  - `README.md` - Project overview and quick links
  - `CLAUDE.md` - Claude Code guidance for this repository (if exists)
  - `spec/ARW-0.1-draft.md` - Editor's draft specification (first 100 lines minimum)
  - `docs/arw-overview/` - Executive summary and publisher control problem
  - `platform/apps/www/` - Demo website showcasing ARW concepts
  - `packages/schemas/README.md` - Schema validation documentation

## Report

Summarize your understanding of:

1. What ARW is and what problem it solves
2. The key components of the ARW v1.0 specification
3. The distinction between technically-enforced actions vs advisory policies
4. The repository structure and purpose of each major directory
5. The discovery architecture (llms.txt + .well-known files + schemas)
6. How the marketing website (`platform/apps/www/`) demonstrates ARW concepts
7. The deployment architecture (Vercel + GitHub Actions)
8. The package management strategy (pnpm for root, npm for platform apps)
9. The `packages/` directory organization and core tooling
10. The CLI tools ecosystem (Rust CLI in `packages/cli/`, validators in `packages/validators/`)
11. The validation pipeline (Python/Node validators, LinkML schemas, GitHub Actions CI)
12. The schema validation system (LinkML + JSON schemas in `packages/schemas/`)
13. The developer tools available (Inspector in `platform/apps/`, DevTools stub in `devtools/`)
14. The example implementations in `examples/` and their different use cases
15. The compliance badge system in `packages/badges/`
16. How to work with the platform apps, packages, examples, schemas, and specification
