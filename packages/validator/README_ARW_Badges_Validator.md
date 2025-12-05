# Agent-Ready Web — Badges & Validator Usage

## Show ARW badges in your README

### Markdown

```md
![Agent-Ready | ARW-1 Discovery](./badges/arw-1-discovery.svg)
![Agent-Ready | ARW-2 Semantic](./badges/arw-2-semantic.svg)
![Agent-Ready | ARW-3 Actions](./badges/arw-3-actions.svg)
![Agent-Ready | ARW-4 Protocol](./badges/arw-4-protocol.svg)
```

### HTML

```html
<img src="/badges/arw-1-discovery.svg" alt="Agent-Ready | ARW-1 Discovery" height="20" />
<img src="/badges/arw-2-semantic.svg" alt="Agent-Ready | ARW-2 Semantic" height="20" />
<img src="/badges/arw-3-actions.svg" alt="Agent-Ready | ARW-3 Actions" height="20" />
<img src="/badges/arw-4-protocol.svg" alt="Agent-Ready | ARW-4 Protocol" height="20" />
```

---

## Validate your ARW implementation

Local validator (Node.js):

```bash
# inside the validator CLI folder
npm i
npm link        # exposes `arw-validate`
arw-validate https://yourdomain.dev
# or without linking
npx arw-validator https://yourdomain.dev
```

What it checks:

- Parses `/llms.txt` (YAML)
- Confirms `version`, `site`, and `policies`
- Requires at least one `content[*].machine_view`
- Fetches a machine view and checks headers:
  - `Content-Type: text/x-llm+markdown`
  - `AI-Attribution`, `AI-Inference`
- If actions exist, basic field sanity
- Reports ARW level (heuristic): ARW-1..ARW-4
