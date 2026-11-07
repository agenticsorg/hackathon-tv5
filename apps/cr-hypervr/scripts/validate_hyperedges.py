#!/usr/bin/env python3
from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Iterable, Iterator

import pandas as pd

# Optional heavy deps are imported lazily where possible to keep startup light


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


async def _ensure_tmp_table(conn) -> None:
    # Create temp table for set comparison (preserve rows across implicit commits)
    await conn.execute(
        """
        CREATE TEMP TABLE IF NOT EXISTS tmp_edges (
          src_kind TEXT,
          src_id BIGINT,
          dst_kind TEXT,
          dst_id BIGINT,
          weight REAL
        ) ON COMMIT PRESERVE ROWS
        """
    )
    # Helpful index for JOIN/NOT EXISTS performance on large edge sets
    await conn.execute(
        "CREATE INDEX IF NOT EXISTS tmp_edges_idx ON tmp_edges(src_kind,src_id,dst_kind,dst_id)"
    )


async def _load_chunk(conn, df: pd.DataFrame) -> int:
    if df is None or df.empty:
        return 0
    part = df[["src_kind", "src_id", "dst_kind", "dst_id", "weight"]].copy()
    rows = list(
        zip(
            part["src_kind"].astype(str),
            part["src_id"].astype(int),
            part["dst_kind"].astype(str),
            part["dst_id"].astype(int),
            part["weight"].astype(float),
        )
    )
    if rows:
        await conn.executemany(
            "INSERT INTO tmp_edges (src_kind, src_id, dst_kind, dst_id, weight) VALUES ($1,$2,$3,$4,$5)",
            rows,
        )
    return len(rows)


def _iter_parquet_batches(parquet_path: str | Path, batch_size: int = 200_000) -> Iterator[pd.DataFrame]:
    """Yield DataFrames with only required columns from a Parquet file.

    Uses pyarrow + fsspec for efficient row-group iteration and low memory use.
    """
    import pyarrow.parquet as pq
    try:
        import fsspec  # provided transitively by gcsfs
    except Exception:  # pragma: no cover
        fsspec = None  # type: ignore

    cols = ["src_kind", "src_id", "dst_kind", "dst_id", "weight"]
    path_str = str(parquet_path)
    if path_str.startswith("gs://") and fsspec is not None:
        with fsspec.open(path_str, "rb") as f:  # type: ignore
            pf = pq.ParquetFile(f)
            for batch in pf.iter_batches(columns=cols, batch_size=batch_size):
                yield batch.to_pandas(types_mapper=None)
    else:
        pf = pq.ParquetFile(path_str)
        for batch in pf.iter_batches(columns=cols, batch_size=batch_size):
            yield batch.to_pandas(types_mapper=None)


async def validate(parquet_path: str, database_url: str, weight_tol: float = 1e-6) -> int:
    import asyncpg

    # Stream parquet into temp table to avoid OOM on large files
    # Also validates required columns exist in the first batch

    conn = await asyncpg.connect(database_url)
    try:
        await _ensure_tmp_table(conn)

        total = 0
        first = True
        for df in _iter_parquet_batches(parquet_path):
            if first:
                first = False
                need = {"src_kind", "src_id", "dst_kind", "dst_id", "weight"}
                if not need.issubset(df.columns):
                    raise RuntimeError(f"Parquet missing columns: {need - set(df.columns)}")
            total += await _load_chunk(conn, df)

        # Count matches/missing with float-tolerant comparison
        # IMPORTANT: use EXISTS to avoid overcount when DB has duplicate rows for a given edge
        q_matched_exists = (
            "SELECT COUNT(*) FROM tmp_edges t WHERE EXISTS ("
            "  SELECT 1 FROM hyperedges h WHERE h.src_kind=t.src_kind AND h.src_id=t.src_id "
            "  AND h.dst_kind=t.dst_kind AND h.dst_id=t.dst_id AND ABS(h.weight - t.weight) < $1"
            ")"
        )
        q_missing = (
            "SELECT COUNT(*) FROM tmp_edges t "
            " WHERE NOT EXISTS ("
            "   SELECT 1 FROM hyperedges h WHERE h.src_kind=t.src_kind AND h.src_id=t.src_id "
            "   AND h.dst_kind=t.dst_kind AND h.dst_id=t.dst_id AND ABS(h.weight - t.weight) < $1"
            " )"
        )
        # Optional: extras present in DB but not in parquet (for debugging)
        q_extra = (
            "SELECT COUNT(*) FROM hyperedges h WHERE NOT EXISTS ("
            "  SELECT 1 FROM tmp_edges t WHERE h.src_kind=t.src_kind AND h.src_id=t.src_id "
            "  AND h.dst_kind=t.dst_kind AND h.dst_id=t.dst_id AND ABS(h.weight - t.weight) < $1"
            ")"
        )
        matched = await conn.fetchval(q_matched_exists, weight_tol)
        missing = await conn.fetchval(q_missing, weight_tol)
        extra = await conn.fetchval(q_extra, weight_tol)
        print({
            "parquet_edges": int(total),
            "db_matched": int(matched or 0),
            "db_missing": int(missing or 0),
            "db_extra": int(extra or 0),
        })
        return 0 if int(missing or 0) == 0 and int(matched or 0) == int(total) else 1
    finally:
        try:
            await conn.execute("DROP TABLE IF EXISTS tmp_edges")
        finally:
            await conn.close()


def main() -> int:
    # Support both local and GCS-style envs
    processed = os.getenv("PROCESSED_PREFIX") or os.getenv("GCS_PROCESSED_PREFIX") or "data/processed"
    parquet_path = (
        f"{processed}/hyperedges.parquet" if str(processed).startswith("gs://") else str(Path(processed) / "hyperedges.parquet")
    )
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        print("Set DATABASE_URL")
        return 2
    # Allow weight tolerance override for float comparisons
    try:
        weight_tol = float(os.getenv("WEIGHT_TOL", "1e-6"))
    except ValueError:
        weight_tol = 1e-6
    import asyncio

    # Quick existence check for clearer error (local or GCS)
    pstr = str(parquet_path)
    if not pstr.startswith("gs://"):
        if not Path(pstr).exists():
            print(f"Missing parquet at {pstr}")
            return 3
    else:
        try:
            import gcsfs  # type: ignore
            fs = gcsfs.GCSFileSystem(token="cloud")
            if not fs.exists(pstr):
                print(f"Missing parquet at {pstr}")
                return 3
        except Exception as e:  # pragma: no cover
            # If we cannot check existence, proceed and let the reader raise an error
            print(f"Warning: couldn't verify GCS path existence for {pstr}: {e}")

    return asyncio.run(validate(parquet_path, db_url, weight_tol=weight_tol))


if __name__ == "__main__":
    sys.exit(main())
