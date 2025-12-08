from __future__ import annotations

import asyncio
import os
from pathlib import Path
import time
from typing import List
import numpy as np
import pandas as pd
import requests


EMBED_DIM = 384
DEFAULT_BATCH = int(os.getenv("BATCH_EMBED_SIZE", "256"))
UPSERT_CHUNK = int(os.getenv("UPSERT_CHUNK_SIZE", "1000"))
DRY_RUN = os.getenv("DRY_RUN", "").lower() in ("1", "true", "yes")


def build_movie_text(row: pd.Series) -> str:
    return f"Title: {row.get('title','')}\nGenres: {row.get('genres','')}\nOverview: {row.get('overview','')}"


async def seed_db(database_url: str, movie_ids: list[int], vectors: np.ndarray) -> None:
    import asyncpg

    conn = await asyncpg.connect(database_url)
    try:
        # Upsert movies and embeddings
        def _vec_to_pgtext(v: list[float]) -> str:
            return "[" + ",".join(str(float(x)) for x in v) + "]"

        q = (
            "INSERT INTO movie_embeddings (movie_id, embedding) "
            "VALUES ($1, $2) ON CONFLICT (movie_id) DO UPDATE SET embedding=EXCLUDED.embedding"
        )
        total = len(movie_ids)
        for i in range(0, total, UPSERT_CHUNK):
            mids = movie_ids[i : i + UPSERT_CHUNK]
            vecs = vectors[i : i + UPSERT_CHUNK]
            rows = [(int(mid), _vec_to_pgtext(vec.tolist())) for mid, vec in zip(mids, vecs)]
            await conn.executemany(q, rows)
            print(f"Upserted {min(i+UPSERT_CHUNK,total)}/{total} embeddings to DB...")
    finally:
        await conn.close()


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


def _encode_vectors_via_service(texts: List[str], batch_size: int, timeout: float = 30.0) -> np.ndarray:
    service_url = os.getenv("SERVICE_URL")
    if not service_url:
        if os.getenv("ALLOW_LOCAL_FALLBACK", "").lower() in ("1", "true", "yes"):
            from app.services.embedder import get_embedder  # type: ignore

            return get_embedder().encode(texts)
        raise SystemExit("SERVICE_URL not set; Cloud Run embedding service required")
    token = _fetch_id_token(service_url)
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    sess = requests.Session()
    out: list[np.ndarray] = []
    for i in range(0, len(texts), batch_size):
        chunk = texts[i : i + batch_size]
        for attempt in range(4):
            try:
                r = sess.post(
                    f"{service_url.rstrip('/')}/embed/batch",
                    json={"texts": chunk},
                    headers=headers,
                    timeout=timeout,
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
    n = np.linalg.norm(arr, axis=1, keepdims=True)
    n[n == 0] = 1.0
    return (arr / n).astype(np.float32)


def _is_gcs(path: str | Path | str) -> bool:
    return str(path).startswith("gs://")


def main():
    processed_env = os.getenv("PROCESSED_PREFIX", "data/processed")
    if _is_gcs(processed_env):
        movies_path = f"{processed_env}/movies_with_descriptions.parquet"
        movies = pd.read_parquet(movies_path, storage_options={"token": "cloud"})
    else:
        processed_dir = Path(processed_env)
        movies_path = processed_dir / "movies_with_descriptions.parquet"
        if not movies_path.exists():
            raise FileNotFoundError("movies_with_descriptions.parquet not found. Run Phase 2 first.")
        movies = pd.read_parquet(movies_path)
    # Align column names
    if "movie_id" not in movies.columns and "movieId" in movies.columns:
        movies = movies.rename(columns={"movieId": "movie_id"})

    # Stream encode + upsert in row chunks to avoid long blocking DB operations
    db_url = os.getenv("DATABASE_URL")
    if not DRY_RUN and not db_url:
        raise SystemExit("DATABASE_URL not set; expected to upsert into movie_embeddings table")

    ROW_CHUNK = int(os.getenv("MOVIES_ROW_CHUNK", "5000"))
    total = len(movies)
    print(f"Processing {total} movies in chunks of {ROW_CHUNK}...")
    processed = 0
    for start in range(0, total, ROW_CHUNK):
        end = min(start + ROW_CHUNK, total)
        chunk = movies.iloc[start:end]
        texts = chunk.apply(build_movie_text, axis=1).tolist()
        mids = chunk["movie_id"].astype(int).tolist()
        print(f"Encoding {len(texts)} movies [{start}:{end}] via service...")
        vecs = _encode_vectors_via_service(texts, batch_size=DEFAULT_BATCH)
        if DRY_RUN:
            print(f"[DRY_RUN] Encoded {len(texts)} embeddings; skipping DB upsert.")
        else:
            print("Upserting chunk to DB...")
            asyncio.run(seed_db(db_url, mids, vecs))
        processed = end
        print(f"Progress: {processed}/{total}")
    print(f"Completed seeding embeddings for {total} movies.")


if __name__ == "__main__":
    main()
