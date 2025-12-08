from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import pandas as pd
from sentence_transformers import InputExample, SentenceTransformer, losses
from torch.utils.data import DataLoader


@dataclass
class TrainConfig:
    base_model_dir: str = os.getenv("BASE_MODEL_DIR", "models/base-minilm")
    output_dir: str = os.getenv("OUTPUT_DIR", "models/movie-minilm-v1")
    processed_dir: str = os.getenv("PROCESSED_DIR", "data/processed")
    epochs: int = int(os.getenv("EPOCHS", 1))
    batch_size: int = int(os.getenv("BATCH_SIZE", 64))
    use_triplet_loss: bool = bool(int(os.getenv("USE_TRIPLET", "0")))


def _read_parquet(path: str):
    storage = {"token": "cloud"} if str(path).startswith("gs://") else None
    return pd.read_parquet(path, storage_options=storage)


def build_examples(processed_dir: str) -> Iterable[InputExample]:
    trips_path = f"{processed_dir}/triplets/triplets_10k.parquet"
    users_path = f"{processed_dir}/user_profiles.parquet"
    # Prefer movies_with_descriptions; allow legacy movies_enriched name
    movies_primary = f"{processed_dir}/movies_with_descriptions.parquet"
    movies_fallback = f"{processed_dir}/movies_enriched.parquet"

    trips = _read_parquet(trips_path)
    users = _read_parquet(users_path)[
        ["user_id", "liked_titles", "disliked_titles"]
    ]
    try:
        movies = _read_parquet(movies_primary)[["movie_id", "title", "overview", "genres"]]
    except Exception:
        movies = _read_parquet(movies_fallback)[["movie_id", "title", "overview", "genres"]]

    u = users.set_index("user_id")
    m = movies.set_index("movie_id")

    def movie_text(mid: int) -> str:
        row = m.loc[mid]
        return f"Title: {row['title']}\nGenres: {row.get('genres', '')}\nOverview: {row.get('overview', '')}"

    for row in trips.itertuples(index=False):
        user_id = int(row.user_id)
        pos = int(row.pos_movie_id)
        neg = int(row.neg_movie_id)
        up = u.loc[user_id]
        anchor = f"User likes: {up.get('liked_titles', '')} \nDislikes: {up.get('disliked_titles', '')}"
        pos_txt = movie_text(pos)
        neg_txt = movie_text(neg)
        yield InputExample(texts=[anchor, pos_txt, neg_txt])


def main():
    cfg = TrainConfig()
    Path(cfg.output_dir).mkdir(parents=True, exist_ok=True)

    print("Loading base model from", cfg.base_model_dir)
    model = SentenceTransformer(cfg.base_model_dir)

    examples = list(build_examples(cfg.processed_dir))
    if not examples:
        raise RuntimeError("No training examples found. Ensure Phase 2 outputs exist.")

    if cfg.use_triplet_loss:
        # Use explicit triplet loss with anchor-pos-neg
        from sentence_transformers.losses import TripletLoss

        train_dataloader = DataLoader(examples, shuffle=True, batch_size=cfg.batch_size)
        train_loss = TripletLoss(model)
    else:
        # Use MNR: Only anchor and positive are used; implicit in-batch negatives apply.
        mnr_examples = [InputExample(texts=e.texts[:2]) for e in examples]
        train_dataloader = DataLoader(mnr_examples, shuffle=True, batch_size=cfg.batch_size)
        train_loss = losses.MultipleNegativesRankingLoss(model)

    print(f"Training for {cfg.epochs} epoch(s) with batch size {cfg.batch_size}")
    model.fit(
        train_objectives=[(train_dataloader, train_loss)],
        epochs=cfg.epochs,
        output_path=cfg.output_dir,
        show_progress_bar=True,
    )
    print("Model saved to", cfg.output_dir)


if __name__ == "__main__":
    main()
