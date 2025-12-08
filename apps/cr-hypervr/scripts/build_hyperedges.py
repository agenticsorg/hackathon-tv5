#!/usr/bin/env python3
from __future__ import annotations

import os
from pathlib import Path
import sys
import gc
from collections import defaultdict
import itertools
import glob
import pandas as pd

try:
    import gcsfs  # type: ignore
except Exception:
    gcsfs = None


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


def _list_ratings_parts(processed_dir: str) -> list[str]:
    pat = f"{processed_dir}/ratings_enriched-*.parquet"
    if str(processed_dir).startswith("gs://"):
        try:
            import gcsfs  # type: ignore
            fs = gcsfs.GCSFileSystem()
            matches = sorted(fs.glob(pat))
            if matches:
                return [m if m.startswith("gs://") else ("gs://" + m) for m in matches]
        except Exception:
            pass
    else:
        files = sorted(glob.glob(pat))
        if files:
            return files
    # Fallback: sequential probe (reader will fail fast if missing)
    return [f"{processed_dir}/ratings_enriched-{i:05d}.parquet" for i in range(0, 200)]


def co_watch_edges(
    df: pd.DataFrame,
    min_rating: float = 4.0,
    max_movies_per_user: int = 20,
    min_pair_count: int = 3,
    top_edges_per_movie: int = 50,
) -> list[tuple[str, int, str, int, float]]:
    """Compute simple co-watch hyperedges between movies watched positively by same user.
    Returns list of (src_kind, src_id, dst_kind, dst_id, weight).
    """
    pos = df[df["rating"] >= min_rating]
    by_user = pos.groupby("user_id")["movie_id"].apply(list)
    counts: dict[tuple[int, int], int] = defaultdict(int)
    for movies in by_user:
        uniq = list(sorted(set(int(m) for m in movies)))
        if len(uniq) > max_movies_per_user:
            uniq = uniq[:max_movies_per_user]
        for a, b in itertools.combinations(uniq, 2):
            counts[(a, b)] += 1
    filtered = {k: v for k, v in counts.items() if v >= min_pair_count}
    per_src: dict[int, list[tuple[int, float]]] = defaultdict(list)
    for (a, b), c in filtered.items():
        per_src[a].append((b, float(c)))
        per_src[b].append((a, float(c)))
    edges: list[tuple[str, int, str, int, float]] = []
    for src, lst in per_src.items():
        lst.sort(key=lambda x: x[1], reverse=True)
        for dst, w in lst[:top_edges_per_movie]:
            edges.append(("movie", src, "movie", dst, w))
    return edges


def genre_affinity_edges(movies: pd.DataFrame) -> list[tuple[str, int, str, int, float]]:
    """Create edges from movie->genre tokens for lightweight hypergraph support."""
    edges: list[tuple[str, int, str, int, float]] = []
    genre_ids: dict[str, int] = {}
    next_gid = 1_000_000  # avoid collision with movie ids
    for row in movies.itertuples(index=False):
        mid = int(getattr(row, "movieId", getattr(row, "movie_id", 0)))
        genres = str(getattr(row, "genres", "")).split("|") if "|" in str(getattr(row, "genres", "")) else str(getattr(row, "genres", "")).split(",")
        for g in [g.strip() for g in genres if g and isinstance(g, str)]:
            gid = genre_ids.setdefault(g, next_gid)
            if gid == next_gid:
                next_gid += 1
            edges.append(("movie", mid, "genre", gid, 1.0))
    return edges


def write_to_db(db_url: str, edges: list[tuple[str, int, str, int, float]]) -> None:
    import asyncpg, asyncio

    async def run() -> None:
        conn = await asyncpg.connect(db_url)
        try:
            await conn.executemany(
                """
                INSERT INTO hyperedges (src_kind, src_id, dst_kind, dst_id, weight)
                VALUES ($1,$2,$3,$4,$5)
                """,
                edges,
            )
        finally:
            await conn.close()

    asyncio.run(run())


def main() -> None:
    # Accept either PROCESSED_PREFIX or legacy GCS_PROCESSED_PREFIX for consistency with other jobs
    processed = os.getenv("PROCESSED_PREFIX") or os.getenv("GCS_PROCESSED_PREFIX") or "data/processed"
    ratings_parts = _list_ratings_parts(processed)
    if not ratings_parts:
        raise FileNotFoundError("ratings_enriched-*.parquet not found in processed dir")
    movies_path = f"{processed}/movies_with_descriptions.parquet" if str(processed).startswith("gs://") else Path(processed) / "movies_with_descriptions.parquet"
    movies = pd.read_parquet(movies_path, storage_options=_storage_options(movies_path))

    # Stream through parts to build co-watch edges without loading all rows in memory
    min_rating = float(os.getenv("MIN_RATING", "4.0"))
    max_movies_per_user = int(os.getenv("MAX_MOVIES_PER_USER", "20"))
    min_pair_count = int(os.getenv("MIN_PAIR_COUNT", "3"))
    top_edges_per_movie = int(os.getenv("TOP_EDGES_PER_MOVIE", "50"))
    max_parts = int(os.getenv("MAX_PARTS", "0"))  # 0 = all
    total_rows = 0
    # Prepare per-part edges output to reduce memory and aid retries
    parts_dir = f"{processed}/hyperedges_parts" if str(processed).startswith("gs://") else str(Path(processed) / "hyperedges_parts")
    if not str(parts_dir).startswith("gs://"):
        Path(parts_dir).mkdir(parents=True, exist_ok=True)

    # Pass 1: generate bounded edges per part and persist
    for idx, p in enumerate(ratings_parts):
        if max_parts and idx >= max_parts:
            break
        try:
            part_df = pd.read_parquet(p, storage_options=_storage_options(p)).copy()
        except FileNotFoundError:
            continue
        if "user_id" not in part_df.columns and "userId" in part_df.columns:
            part_df = part_df.rename(columns={"userId": "user_id"})
        if "movie_id" not in part_df.columns and "movieId" in part_df.columns:
            part_df = part_df.rename(columns={"movieId": "movie_id"})
        total_rows += len(part_df)
        part_edges = co_watch_edges(
            part_df,
            min_rating=min_rating,
            max_movies_per_user=max_movies_per_user,
            min_pair_count=min_pair_count,
            top_edges_per_movie=top_edges_per_movie,
        )
        # Persist part edges
        edf = pd.DataFrame(part_edges, columns=["src_kind", "src_id", "dst_kind", "dst_id", "weight"]) if part_edges else pd.DataFrame(columns=["src_kind", "src_id", "dst_kind", "dst_id", "weight"])
        outp = f"{parts_dir}/edges_part_{idx:05d}.parquet" if str(parts_dir).startswith("gs://") else str(Path(parts_dir) / f"edges_part_{idx:05d}.parquet")
        edf.to_parquet(outp, storage_options=_storage_options(outp), index=False)
        print(f"Wrote edges for part {idx:05d}: {len(edf):,} -> {outp}")
        sys.stdout.flush()
        # Free memory between parts
        del part_df, part_edges, edf
        gc.collect()

    # Pass 2: aggregate per-part edges into bounded top-K per source
    from glob import glob as _glob
    parts_list: list[str]
    if str(parts_dir).startswith("gs://"):
        try:
            import gcsfs  # type: ignore
            fs = gcsfs.GCSFileSystem()
            parts_list = sorted(fs.glob(f"{parts_dir}/edges_part_*.parquet"))
            parts_list = [p if p.startswith("gs://") else ("gs://" + p) for p in parts_list]
        except Exception:
            parts_list = []
    else:
        parts_list = sorted(_glob(str(Path(parts_dir) / "edges_part_*.parquet")))

    per_src_global: dict[int, dict[int, float]] = defaultdict(dict)
    for j, ep in enumerate(parts_list):
        try:
            e = pd.read_parquet(ep, storage_options=_storage_options(ep), columns=["src_id", "dst_id", "weight"])  # type: ignore[arg-type]
        except Exception:
            continue
        for row in e.itertuples(index=False):
            src = int(getattr(row, "src_id"))
            dst = int(getattr(row, "dst_id"))
            w = float(getattr(row, "weight"))
            d = per_src_global[src]
            d[dst] = d.get(dst, 0.0) + w
        # Prune per-src maps to keep bounded size
        for src, d in list(per_src_global.items()):
            if len(d) > top_edges_per_movie:
                top = sorted(d.items(), key=lambda kv: kv[1], reverse=True)[:top_edges_per_movie]
                per_src_global[src] = dict(top)
        if (j + 1) % 10 == 0:
            approx_edges = sum(len(d) for d in per_src_global.values())
            print(f"Aggregated {j+1}/{len(parts_list)} edge parts, approx edges {approx_edges:,}")
            sys.stdout.flush()

    # Emit final edges
    edges: list[tuple[str, int, str, int, float]] = []
    for src, d in per_src_global.items():
        for dst, w in d.items():
            edges.append(("movie", src, "movie", dst, float(w)))
    edges += genre_affinity_edges(movies)

    # Write optional parquet artifact (local or GCS)
    out_path = f"{processed}/hyperedges.parquet" if str(processed).startswith("gs://") else str(Path(processed) / "hyperedges.parquet")
    pd.DataFrame(edges, columns=["src_kind", "src_id", "dst_kind", "dst_id", "weight"]).to_parquet(out_path, storage_options=_storage_options(out_path), index=False)
    print(f"Hyperedges written to {out_path} ({len(edges):,} rows)")

    db_url = os.getenv("DATABASE_URL")
    if db_url:
        # Chunked DB insert to avoid long blocking executemany
        CHUNK = int(os.getenv("EDGE_DB_CHUNK", "5000"))
        import asyncpg, asyncio
        async def run() -> None:
            conn = await asyncpg.connect(db_url)
            try:
                q = "INSERT INTO hyperedges (src_kind, src_id, dst_kind, dst_id, weight) VALUES ($1,$2,$3,$4,$5)"
                for i in range(0, len(edges), CHUNK):
                    await conn.executemany(q, edges[i:i+CHUNK])
                    print(f"Inserted edges {min(i+CHUNK, len(edges))}/{len(edges)} into DB...")
                    sys.stdout.flush()
            finally:
                await conn.close()
        asyncio.run(run())
        print("Also inserted into database hyperedges table.")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
