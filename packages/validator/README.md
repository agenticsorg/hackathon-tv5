# ARW Validator CLI

Validate an Agent-Ready Web implementation for a given domain.

## Install (local)

```bash
npm i
npm link
```

## Usage

```bash
arw-validate https://example.com
# or
npx arw-validator https://example.com
```

## What it checks

- Fetches `/llms.txt` and parses YAML
- Ensures `version` and `profile` are present
- Validates required sections: `site`, `policies`
- Checks at least one `content[*].machine_view`
- Requests one machine view and verifies headers:
  - `AI-Attribution` (present)
  - `AI-Inference` (present)
  - `Content-Type: text/x-llm+markdown`
- If `actions` exist, verifies OAuth endpoints are reachable
- Prints a conformance summary (ARW-1..4) based on manifest
