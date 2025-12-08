from __future__ import annotations

from typing import Dict, List


def combine_scores(
    base_scores: Dict[int, float],
    genre_weights: Dict[int, float],
    weight: float = 0.05,
) -> Dict[int, float]:
    """
    Combine cosine similarity scores with simple graph-derived weights.
    For now: new = base + weight * genre_weight(movie_id).
    """
    out: Dict[int, float] = {}
    for mid, s in base_scores.items():
        g = genre_weights.get(mid, 0.0)
        out[mid] = float(s) + float(weight) * float(g)
    return out


def reorder_by_scores(items: List[dict], scores: Dict[int, float]) -> List[dict]:
    return sorted(items, key=lambda it: scores.get(int(it.get("movie_id")), 0.0), reverse=True)

