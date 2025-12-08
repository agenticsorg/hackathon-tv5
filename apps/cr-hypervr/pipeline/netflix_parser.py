from __future__ import annotations

import re
from pathlib import Path
import pandas as pd
from tqdm import tqdm


MOVIE_HEADER_RE = re.compile(r"^(\d+):\s*$")


def parse_combined_files(netflix_dir: Path) -> pd.DataFrame:
    files = [
        netflix_dir / "combined_data_1.txt",
        netflix_dir / "combined_data_2.txt",
        netflix_dir / "combined_data_3.txt",
        netflix_dir / "combined_data_4.txt",
    ]
    rows: list[tuple[int, int, int, str]] = []  # movie_id, user_id, rating, date
    for f in files:
        if not f.exists():
            continue
        movie_id = None
        with f.open("r", encoding="latin-1") as fh:
            for line in fh:
                m = MOVIE_HEADER_RE.match(line)
                if m:
                    movie_id = int(m.group(1))
                    continue
                if movie_id is None:
                    continue
                parts = line.strip().split(",")
                if len(parts) != 3:
                    continue
                user_id, rating, date = int(parts[0]), int(parts[1]), parts[2]
                rows.append((movie_id, user_id, rating, date))

    df = pd.DataFrame(rows, columns=["movie_id", "user_id", "rating", "date"])
    return df


def load_movie_titles(netflix_dir: Path) -> pd.DataFrame:
    mt = netflix_dir / "movie_titles.csv"
    if not mt.exists():
        raise FileNotFoundError("movie_titles.csv not found")
    # movie_id, year, title
    df = pd.read_csv(mt, header=None, names=["movie_id", "year", "title"], encoding="latin-1")
    return df


def build(netflix_dir: str = "data/netflix", out_dir: str = "data/processed") -> None:
    netflix = Path(netflix_dir)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)

    print("Parsing Netflix combined data files...")
    ratings = parse_combined_files(netflix)
    print(f"Parsed {len(ratings):,} ratings")
    ratings.to_parquet(out / "ratings.parquet")

    print("Loading movie titles...")
    movies = load_movie_titles(netflix)
    movies.to_parquet(out / "movies.parquet")

    # Basic validation stats
    print(
        {
            "users": ratings["user_id"].nunique(),
            "movies": ratings["movie_id"].nunique(),
            "ratings": len(ratings),
        }
    )


if __name__ == "__main__":
    build()

