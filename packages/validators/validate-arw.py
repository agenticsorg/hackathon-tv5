#!/usr/bin/env python3
import argparse, sys, json, yaml
from jsonschema import validate, Draft7Validator
from jsonschema.exceptions import ValidationError

def main():
    ap = argparse.ArgumentParser(description="Validate ARW /llms.txt against ARW JSON Schema")
    ap.add_argument("manifest", help="Path or URL to llms.txt/llms.yaml (YAML)")
    ap.add_argument("--schema", default="schemas/arw_model.json", help="Path to ARW JSON Schema")
    args = ap.parse_args()

    # Load schema
    with open(args.schema, "r") as f:
        schema = json.load(f)

    # Load YAML
    with open(args.manifest, "r") as f:
        data = yaml.safe_load(f)

    v = Draft7Validator(schema)
    errors = sorted(v.iter_errors(data), key=lambda e: e.path)
    if errors:
        print("❌ Manifest failed validation:")
        for e in errors:
            path = "/".join([str(p) for p in e.path])
            print(f"  - {path}: {e.message}")
        sys.exit(1)
    else:
        print("✅ Manifest is valid.")
        sys.exit(0)

if __name__ == "__main__":
    main()
