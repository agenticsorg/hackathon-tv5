#!/usr/bin/env python3
from __future__ import annotations

"""
Backfill movie embeddings directly from the Cloud SQL database.

Reads rows from `movies` that are missing an entry in `movie_embeddings`,
builds a simple text representation (title/genres/overview), encodes with the
service embedder (SentenceTransformer or hash backend), and upserts into DB.

Environment:
- DATABASE_URL (required)
- BATCH_SIZE (optional, default 256)
- LIMIT (optional, limit number of rows for test runs)
- EMBEDDING_BACKEND (optional, e.g., st|hash|auto)

Intended to run inside the same container image used by the API and Cloud Run
Jobs, so it has the same dependencies and cached base MiniLM model.
"""

import asyncio
import os
from typing import Iterable, List, Tuple
import time
import requests

import numpy as np


def build_movie_text(title: str | None, genres: str | None, overview: str | None) -> str:
    return (
        f"Title: {title or ''}\n"
        f"Genres: {genres or ''}\n"
        f"Overview: {overview or ''}"
    )


def _fetch_id_token(audience: str) -> str | None:
    tok = os.getenv("ID_TOKEN")
    if tok:
        return tok
    try:
        resp = requests.get(
            "http://metadata/computeMetadata/v1/instance/service-accounts/default/identity",
            params={"audience": audience, "format": "full"},
            headers={"Metadata-Flavor": "Google"},
            timeout=3,
        )
        if resp.status_code == 200 and resp.text:
            return resp.text.strip()
    except Exception:
        pass
    return None


def _encode(texts: List[str]) -> np.ndarray:
    service_url = os.getenv("SERVICE_URL")
    if not service_url:
        if os.getenv("ALLOW_LOCAL_FALLBACK", "").lower() in ("1", "true", "yes"):
            from app.services.embedder import get_embedder  # type: ignore

            vecs = get_embedder().encode(texts)
            vecs = vecs.astype(np.float32)
            norms = np.linalg.norm(vecs, axis=1, keepdims=True)
            norms[norms == 0] = 1.0
            return vecs / norms
        raise SystemExit("SERVICE_URL not set; Cloud Run embedding service required")
    token = _fetch_id_token(service_url)
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    sess = requests.Session()
    out: list[np.ndarray] = []
    batch = int(os.getenv("BATCH_EMBED_SIZE", "256"))
    for i in range(0, len(texts), batch):
        chunk = texts[i : i + batch]
        for attempt in range(4):
            try:
                r = sess.post(
                    f"{service_url.rstrip('/')}/embed/batch",
                    json={"texts": chunk},
                    headers=headers,
                    timeout=30,
                )
                if r.status_code >= 500 and attempt < 3:
                    time.sleep(1.5 * (attempt + 1))
                    continue
                r.raise_for_status()
                payload = r.json()
                vecs = [np.array(item["embedding"], dtype=np.float32) for item in payload]
                out.extend(vecs)
                break
            except Exception:
                if attempt >= 3:
                    raise
                time.sleep(1.5 * (attempt + 1))
                continue
    arr = np.stack(out, axis=0)
    norms = np.linalg.norm(arr, axis=1, keepdims=True)
    norms[norms == 0] = 1.0
    return (arr / norms).astype(np.float32)


def _vec_to_pg(v: Iterable[float]) -> str:
    return "[" + ",".join(str(float(x)) for x in v) + "]"


async def _fetch_missing(conn, limit: int | None) -> List[Tuple[int, str | None, str | None, str | None]]:
    q = (
        "SELECT m.movie_id, m.title, m.genres, m.overview "
        "FROM movies m LEFT JOIN movie_embeddings e USING (movie_id) "
        "WHERE e.movie_id IS NULL ORDER BY m.movie_id"
    )
    if limit and limit > 0:
        q += " LIMIT $1"
        rows = await conn.fetch(q, int(limit))
    else:
        rows = await conn.fetch(q)
    return [(int(r["movie_id"]), r["title"], r["genres"], r["overview"]) for r in rows]


async def _upsert(conn, mids: List[int], vecs: np.ndarray) -> None:
    await conn.executemany(
        (
            "INSERT INTO movie_embeddings (movie_id, embedding) "
            "VALUES ($1, $2) ON CONFLICT (movie_id) DO UPDATE SET embedding=EXCLUDED.embedding"
        ),
        [(int(mid), _vec_to_pg(vec.tolist())) for mid, vec in zip(mids, vecs)],
    )


async def backfill(database_url: str, batch_size: int = 256, limit: int | None = None) -> int:
    import asyncpg  # lazy import to keep import-time deps light for tests

    conn = await asyncpg.connect(database_url)
    processed = 0
    try:
        pending = await _fetch_missing(conn, limit)
        if not pending:
            print("No missing embeddings found.")
            return 0
        print(f"Missing embeddings: {len(pending)}")
        # Process in batches
        for i in range(0, len(pending), batch_size):
            batch = pending[i : i + batch_size]
            mids = [mid for (mid, _t, _g, _o) in batch]
            texts = [build_movie_text(t, g, o) for (_mid, t, g, o) in batch]
            vecs = _encode(texts)
            await _upsert(conn, mids, vecs)
            processed += len(batch)
            print(f"Upserted {processed}/{len(pending)} embeddings...")
        return processed
    finally:
        await conn.close()


def main() -> int:
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        print("Set DATABASE_URL", flush=True)
        return 2
    batch_size = int(os.getenv("BATCH_SIZE", "256"))
    limit_env = os.getenv("LIMIT")
    limit = int(limit_env) if limit_env else None
    print(f"Starting backfill: batch_size={batch_size}, limit={limit}")
    processed = asyncio.run(backfill(db_url, batch_size=batch_size, limit=limit))
    print(f"Backfill complete. Processed: {processed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
