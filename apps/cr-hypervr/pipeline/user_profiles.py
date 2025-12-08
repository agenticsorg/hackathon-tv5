from __future__ import annotations

from pathlib import Path
import os
import pandas as pd
from collections import defaultdict
from typing import Dict, List
import glob
try:
    import gcsfs  # noqa: F401
except Exception:
    gcsfs = None


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


def _list_ratings_parts(processed_dir: str) -> List[str]:
    pat = f"{processed_dir}/ratings_enriched-*.parquet"
    # Try gcsfs glob first
    if processed_dir.startswith("gs://"):
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
    # Fallback: sequential probe up to 200 parts
    out: List[str] = []
    for i in range(0, 200):
        p = f"{processed_dir}/ratings_enriched-{i:05d}.parquet"
        # Defer existence check to reader; caller will catch FileNotFoundError
        out.append(p)
    return out


def build_user_profiles(
    processed_dir: str = os.getenv("GCS_PROCESSED_PREFIX", "data/processed"),
    out_path: str = os.getenv("GCS_PROFILES_PATH", "data/processed/user_profiles.parquet"),
    min_ratings: int = 10,
) -> None:
    # Stream-friendly aggregation across enriched parts
    parts = _list_ratings_parts(processed_dir)
    if not parts:
        # Fallback to single-file path
        single = f"{processed_dir}/ratings_enriched.parquet"
        parts = [single]

    counts: Dict[int, int] = defaultdict(int)
    pos_titles: Dict[int, List[str]] = defaultdict(list)
    neg_titles: Dict[int, List[str]] = defaultdict(list)

    def _cap_append(d: Dict[int, List[str]], k: int, vals: List[str], cap: int = 50) -> None:
        if not vals:
            return
        cur = d[k]
        room = cap - len(cur)
        if room <= 0:
            return
        cur.extend([v for v in vals[:room] if isinstance(v, str)])

    for p in parts:
        df = pd.read_parquet(p, storage_options=_storage_options(p), columns=["user_id", "rating", "title"])
        # counts
        for uid, n in df.groupby("user_id").size().items():
            counts[int(uid)] += int(n)
        # positives
        pos = df[df["rating"] >= 4.0]
        if not pos.empty:
            agg = pos.groupby("user_id")["title"].apply(lambda s: list(s.dropna().astype(str))).to_dict()
            for uid, titles in agg.items():
                _cap_append(pos_titles, int(uid), titles)
        # negatives
        neg = df[df["rating"] <= 2.0]
        if not neg.empty:
            agg = neg.groupby("user_id")["title"].apply(lambda s: list(s.dropna().astype(str))).to_dict()
            for uid, titles in agg.items():
                _cap_append(neg_titles, int(uid), titles)

    # Build final DataFrame
    rows = []
    for uid, cnt in counts.items():
        if cnt < min_ratings:
            continue
        rows.append(
            {
                "user_id": uid,
                "num_ratings": int(cnt),
                "liked_titles": ", ".join(pos_titles.get(uid, [])[:50]),
                "disliked_titles": ", ".join(neg_titles.get(uid, [])[:50]),
            }
        )
    profiles = pd.DataFrame(rows)
    profiles.to_parquet(out_path, storage_options=_storage_options(out_path), index=False)
    print(f"User profiles saved to {out_path} ({len(profiles):,} users)")


if __name__ == "__main__":
    build_user_profiles()
