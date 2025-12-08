#!/usr/bin/env python3
from __future__ import annotations

import asyncio
import os
import sys
from typing import Optional

import asyncpg


SQL_FILES = [
    "${GCS_EMB_BUCKET}/db/pgvector.sql",
    "${GCS_EMB_BUCKET}/db/schema.sql",
]


async def run_sql(conn: asyncpg.Connection, sql_text: str) -> None:
    # asyncpg can execute multiple statements in one call
    await conn.execute(sql_text)


async def load_gcs_text(path: str) -> str:
    import gcsfs  # lazy import

    fs = gcsfs.GCSFileSystem()
    with fs.open(path, "r") as f:
        return f.read()


async def main() -> int:
    db_url = os.getenv("DATABASE_URL")
    gcs_bucket = os.getenv("GCS_EMB_BUCKET")
    if not db_url or not gcs_bucket:
        print("Missing env: DATABASE_URL or GCS_EMB_BUCKET", file=sys.stderr)
        return 2

    # Resolve file paths with env substitution
    files = [p.replace("${GCS_EMB_BUCKET}", gcs_bucket) for p in SQL_FILES]
    print("Applying SQL files:", files)
    conn: Optional[asyncpg.Connection] = None
    try:
        conn = await asyncpg.connect(dsn=db_url)
        for p in files:
            try:
                sql_text = await load_gcs_text(p)
                print(f"-- Executing: {p} ({len(sql_text)} bytes)")
                await run_sql(conn, sql_text)
                print(f"OK: {p}")
            except Exception as e:
                print(f"ERROR executing {p}: {e}", file=sys.stderr)
                return 1
        return 0
    finally:
        if conn:
            await conn.close()


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))

