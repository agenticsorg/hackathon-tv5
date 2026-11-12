from __future__ import annotations

from functools import lru_cache
from typing import Sequence


class Reranker:
    """
    Tiny, optional reranker stub. Deterministic and lightweight so it can
    be enabled in production without heavy deps. Intended as a seam where
    a true cross-encoder (TinyBERT, etc.) could be integrated later.
    """

    def rerank(self, query: str, items: Sequence[dict]) -> list[dict]:
        # Heuristic: prefer titles that share tokens with the query (case-insensitive),
        # stable sort to keep original ranking when scores tie.
        q_tokens = {t for t in query.lower().split() if t}

        def score(it: dict) -> int:
            title = str(it.get("title", "")).lower()
            tokens = set(title.split())
            return len(q_tokens & tokens)

        return sorted(list(items), key=score, reverse=True)


@lru_cache(maxsize=1)
def get_reranker() -> Reranker:
    return Reranker()

