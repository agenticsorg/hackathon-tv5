#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from fastapi.openapi.utils import get_openapi
from app.main import app


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--out", default="docs/openapi.json", help="Output file path")
    args = p.parse_args()
    schema = get_openapi(
        title=app.title,
        version=app.version,
        routes=app.routes,
        description="Movie Embedding Service OpenAPI",
    )
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(schema, indent=2))
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()

