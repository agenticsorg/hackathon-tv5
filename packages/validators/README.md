# ARW Validators

This directory contains validation tools for ARW manifests and discovery files.

## Python Validator

Validates ARW `llms.txt` files against the JSON Schema.

### Requirements

```bash
pip install jsonschema pyyaml
```

### Usage

```bash
# Validate a manifest
python tools/validators/validate-arw.py www/public/llms.txt --schema schemas/arw_model.json

# Or make it executable
chmod +x tools/validators/validate-arw.py
./tools/validators/validate-arw.py www/public/llms.txt
```

## Node.js Validator

JavaScript/TypeScript alternative using Ajv.

### Requirements

```bash
npm install ajv ajv-formats yaml
```

### Usage

```bash
# Validate a manifest
node tools/validators/validate-arw.mjs www/public/llms.txt schemas/arw_model.json

# Or make it executable
chmod +x tools/validators/validate-arw.mjs
./tools/validators/validate-arw.mjs www/public/llms.txt schemas/arw_model.json
```

## What Gets Validated

Both validators check:

- ✅ Required fields (version, profile, site, policies)
- ✅ Profile values (ARW-1, ARW-2, ARW-3, ARW-4)
- ✅ URI formats for URLs
- ✅ Email format for contact
- ✅ Content item structure (url, machine_view, chunks)
- ✅ Action definitions (endpoint, method, auth)
- ✅ OAuth configuration
- ✅ Policy declarations (training, inference, attribution)

## Integration with CI

See `.github/workflows/validate-arw.yml` for GitHub Actions integration.
