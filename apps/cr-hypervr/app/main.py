from fastapi import FastAPI, Depends, Response
from app.core.config import settings
from app import __version__
from app.schemas import (
    EmbedTextRequest,
    EmbedBatchRequest,
    EmbedVectorResponse,
    SimilarSearchRequest,
    SimilarSearchResponse,
    SimilarItem,
    RecommendRequest,
    RecommendResponse,
    MovieEmbedRequest,
    UserEmbedRequest,
    GraphRecommendRequest,
    GraphRecommendResponse,
    GraphRecommendItem,
)
from app.services import embedder as embedder_service
from app.services.reranker import get_reranker
from app.services.scoring import combine_scores, reorder_by_scores
from typing import TYPE_CHECKING
import numpy as np
from fastapi.middleware.cors import CORSMiddleware
from fastapi import HTTPException

# Simple in-memory metrics
_metrics = {
    "requests_total": 0,
    "embed_text_total": 0,
    "embed_batch_total": 0,
    "embed_movie_total": 0,
    "embed_user_total": 0,
    "search_similar_total": 0,
    "search_recommend_total": 0,
    "graph_recommend_total": 0,
}


if TYPE_CHECKING:  # for type checkers only
    from app.db.client import DB  # pragma: no cover


def _get_db_dep():
    # Lazy import; tolerate missing asyncpg in environments without DB
    try:
        from app.db.client import get_db  # type: ignore
        return get_db()
    except Exception:
        class _NoDB:
            async def connect(self):
                raise RuntimeError("DATABASE_URL not configured or driver not installed")

        return _NoDB()


def create_app() -> FastAPI:
    app = FastAPI(title=settings.app_name, version=__version__)

    # CORS
    if settings.allowed_origins:
        app.add_middleware(
            CORSMiddleware,
            allow_origins=settings.allowed_origins,
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )

    @app.get("/healthz")
    async def healthz():
        return {"status": "ok"}

    @app.get("/")
    async def root():
        return {
            "service": settings.app_name,
            "version": __version__,
            "environment": settings.environment,
        }

    @app.post("/embed/text", response_model=EmbedVectorResponse)
    async def embed_text(payload: EmbedTextRequest):
        _metrics["requests_total"] += 1
        _metrics["embed_text_total"] += 1
        vec = embedder_service.get_embedder().encode([payload.text])[0]
        return {
            "embedding": vec.tolist(),
            "dimension": settings.vector_dim,
            "model": settings.model_name,
        }

    @app.post("/embed/batch", response_model=list[EmbedVectorResponse])
    async def embed_batch(payload: EmbedBatchRequest):
        _metrics["requests_total"] += 1
        _metrics["embed_batch_total"] += 1
        vecs = embedder_service.get_embedder().encode(payload.texts)
        return [
            {
                "embedding": v.tolist(),
                "dimension": settings.vector_dim,
                "model": settings.model_name,
            }
            for v in vecs
        ]

    @app.post("/embed/movie", response_model=EmbedVectorResponse)
    async def embed_movie(payload: MovieEmbedRequest):
        _metrics["requests_total"] += 1
        _metrics["embed_movie_total"] += 1
        genres = ", ".join(payload.genres) if payload.genres else ""
        desc = (payload.description or "").strip()
        if len(desc) > 500:
            desc = desc[:500]
        text = f"{payload.title}. {genres}. {desc}"
        vec = embedder_service.get_embedder().encode([text])[0]
        return {
            "embedding": vec.tolist(),
            "dimension": settings.vector_dim,
            "model": settings.model_name,
        }

    @app.post("/embed/user", response_model=EmbedVectorResponse)
    async def embed_user(payload: UserEmbedRequest):
        _metrics["requests_total"] += 1
        _metrics["embed_user_total"] += 1
        likes = ", ".join(payload.liked_movies)
        top_genres = ", ".join(payload.liked_genres)
        dislikes = ", ".join(payload.disliked_genres)
        text = f"Enjoys {top_genres}. Liked movies such as {likes}. Avoids {dislikes}."
        vec = embedder_service.get_embedder().encode([text])[0]
        return {
            "embedding": vec.tolist(),
            "dimension": settings.vector_dim,
            "model": settings.model_name,
        }

    @app.post("/search/similar", response_model=SimilarSearchResponse)
    async def search_similar(payload: SimilarSearchRequest, db = Depends(_get_db_dep)):
        _metrics["requests_total"] += 1
        _metrics["search_similar_total"] += 1
        query_vec = embedder_service.get_embedder().encode([payload.text])[0]
        items = await db.fetch_similar(query_vec.astype(np.float32), top_k=payload.top_k)
        # Optional graph-based scoring using genre hyperedges
        if getattr(settings, "use_graph_scorer", False) and items:
            mids = [int(i["movie_id"]) for i in items]
            gweights = await db.fetch_genre_weights(mids)
            base = {int(i["movie_id"]): float(i.get("score", 0.0)) for i in items}
            scores = combine_scores(base, gweights, weight=getattr(settings, "graph_score_weight", 0.05))
            items = reorder_by_scores(items, scores)
        # Optional rerank
        if settings.use_reranker and items:
            items = get_reranker().rerank(payload.text, items)
        return {
            "items": [
                SimilarItem(movie_id=i["movie_id"], title=i["title"], genres=i.get("genres"), score=float(i["score"]))
                for i in items
            ]
        }

    @app.post("/search/recommend", response_model=RecommendResponse)
    async def recommend(payload: RecommendRequest, db = Depends(_get_db_dep)):
        _metrics["requests_total"] += 1
        _metrics["search_recommend_total"] += 1
        # Prefer DB-derived user profile embedding (avg of liked items)
        vec_np = await db.fetch_user_profile_embedding(payload.user_id)
        if vec_np is None:
            # Fallback: encode a deterministic user token
            text = f"user_id:{payload.user_id}"
            vec_np = embedder_service.get_embedder().encode([text])[0].astype(np.float32)
        vec = vec_np.astype(np.float32)
        items = await db.fetch_similar(vec, top_k=payload.top_k + (len(payload.exclude_movie_ids) if payload.exclude_movie_ids else 0))
        exclude = set(payload.exclude_movie_ids or [])
        filtered = [i for i in items if i["movie_id"] not in exclude]
        # Optional graph-based scoring
        if getattr(settings, "use_graph_scorer", False) and filtered:
            mids = [int(i["movie_id"]) for i in filtered]
            gweights = await db.fetch_genre_weights(mids)
            base = {int(i["movie_id"]): float(i.get("score", 0.0)) for i in filtered}
            scores = combine_scores(base, gweights, weight=getattr(settings, "graph_score_weight", 0.05))
            filtered = reorder_by_scores(filtered, scores)
        # Optional rerank
        if settings.use_reranker and filtered:
            filtered = get_reranker().rerank("user profile", filtered)
        filtered = filtered[: payload.top_k]
        return {
            "items": [
                SimilarItem(movie_id=i["movie_id"], title=i["title"], genres=i.get("genres"), score=float(i["score"]))
                for i in filtered
            ]
        }

    @app.post("/graph/recommend", response_model=GraphRecommendResponse)
    async def graph_recommend(payload: GraphRecommendRequest, db = Depends(_get_db_dep)):
        """Embed free‑text query, seed via vector search, expand through hypergraph, and return recommendations.

        Expansion rules:
        - hops>=1 → co‑watch neighbors (movie→movie)
        - hops>=2 → shared‑genre neighbors (movie→genre→movie)
        Scores are normalized per‑signal and linearly combined using weights.
        """
        _metrics["requests_total"] += 1
        _metrics["graph_recommend_total"] += 1

        # Defensive: ensure DB has expected API
        for need in ("fetch_similar", "fetch_neighbors_cowatch", "fetch_neighbors_shared_genre", "fetch_movies_by_ids"):
            if not hasattr(db, need):
                raise HTTPException(status_code=503, detail="Database not configured")

        # 1) Seed via vector search
        query_vec = embedder_service.get_embedder().encode([payload.query])[0]
        seeds = await db.fetch_similar(query_vec.astype(np.float32), top_k=max(payload.seed_top_k, payload.top_k))
        seed_ids = [int(s["movie_id"]) for s in seeds]
        embed_scores = {int(s["movie_id"]): float(s.get("score", 0.0)) for s in seeds}

        # 2) Graph expansion signals
        cowatch: dict[int, float] = {}
        by_genre: dict[int, float] = {}
        if payload.hops >= 1 and seed_ids:
            cowatch = await db.fetch_neighbors_cowatch(seed_ids, top_k=max(3 * payload.seed_top_k, 200))
        if payload.hops >= 2 and seed_ids:
            by_genre = await db.fetch_neighbors_shared_genre(seed_ids, top_k=max(5 * payload.seed_top_k, 400))

        # 3) Normalize each signal to [0,1] for stable mixing
        def _normalize(d: dict[int, float]) -> dict[int, float]:
            if not d:
                return {}
            m = max(d.values())
            if m <= 0:
                return {k: 0.0 for k in d}
            return {k: float(v) / float(m) for k, v in d.items()}

        embed_n = _normalize(embed_scores)
        cowatch_n = _normalize(cowatch)
        genre_n = _normalize(by_genre)

        # 4) Aggregate combined scores; exclude seed items for recommendations
        combined: dict[int, dict[str, float]] = {}
        keys = set(embed_n) | set(cowatch_n) | set(genre_n)
        for mid in keys:
            if mid in seed_ids:
                continue
            e = embed_n.get(mid, 0.0)
            c = cowatch_n.get(mid, 0.0)
            g = genre_n.get(mid, 0.0)
            score = payload.embed_weight * e + payload.cowatch_weight * c + payload.genre_weight * g
            if score <= 0:
                continue
            combined[mid] = {"score": score, "e": e, "c": c, "g": g}

        ranked = sorted(combined.items(), key=lambda kv: kv[1]["score"], reverse=True)
        ranked = ranked[: payload.top_k]
        mids = [mid for mid, _ in ranked]
        meta = await db.fetch_movies_by_ids(mids)

        def _sources(sig: dict[str, float]) -> list[str]:
            out: list[str] = []
            if sig.get("e", 0) > 0:
                out.append("embed")
            if sig.get("c", 0) > 0:
                out.append("cowatch")
            if sig.get("g", 0) > 0:
                out.append("genre")
            return out

        items = []
        for mid, sig in ranked:
            m = meta.get(mid, {})
            items.append(
                GraphRecommendItem(
                    movie_id=mid,
                    title=m.get("title", str(mid)),
                    genres=m.get("genres"),
                    score=float(sig["score"]),
                    sources=_sources(sig) or None,
                )
            )
        return {"items": items}

    @app.get("/ready")
    async def ready() -> dict:
        # Basic readiness check (lightweight)
        try:
            _ = embedder_service.get_embedder()
            return {"ready": True}
        except Exception:
            return {"ready": False}

    @app.get("/metrics")
    async def metrics() -> Response:
        lines = [
            "# HELP service_requests_total Total HTTP requests.",
            "# TYPE service_requests_total counter",
            f"service_requests_total {_metrics['requests_total']}",
            "# HELP embed_requests_total Total embed requests by type.",
            "# TYPE embed_requests_total counter",
            f"embed_requests_total{{type=\"text\"}} {_metrics['embed_text_total']}",
            f"embed_requests_total{{type=\"batch\"}} {_metrics['embed_batch_total']}",
            f"embed_requests_total{{type=\"movie\"}} {_metrics['embed_movie_total']}",
            f"embed_requests_total{{type=\"user\"}} {_metrics['embed_user_total']}",
            "# HELP search_requests_total Total search requests by type.",
            "# TYPE search_requests_total counter",
            f"search_requests_total{{type=\"similar\"}} {_metrics['search_similar_total']}",
            f"search_requests_total{{type=\"recommend\"}} {_metrics['search_recommend_total']}",
            f"search_requests_total{{type=\"graph_recommend\"}} {_metrics['graph_recommend_total']}",
        ]
        return Response("\n".join(lines) + "\n", media_type="text/plain; version=0.0.4")

    # Debug endpoints (safe for demos; avoid leaking secrets). These endpoints
    # do not raise if DB is unavailable; they return availability flags.
    @app.get("/debug/db_counts")
    async def db_counts(db = Depends(_get_db_dep)):
        try:
            # Count movies and embeddings
            await db.connect()
            conn = db._pool  # type: ignore[attr-defined]
            assert conn is not None
            async with conn.acquire() as c:  # type: ignore
                m = await c.fetchval("SELECT COUNT(*) FROM movies")
                e = await c.fetchval("SELECT COUNT(*) FROM movie_embeddings")
            return {"available": True, "movies": int(m or 0), "embeddings": int(e or 0)}
        except Exception:
            # DB not configured or unreachable
            return {"available": False, "movies": 0, "embeddings": 0}

    @app.get("/debug/sample_movie")
    async def sample_movie(db = Depends(_get_db_dep)):
        try:
            await db.connect()
            conn = db._pool  # type: ignore[attr-defined]
            assert conn is not None
            async with conn.acquire() as c:  # type: ignore
                row = await c.fetchrow("SELECT movie_id, title FROM movies LIMIT 1")
            if row:
                return {"available": True, "movie": {"movie_id": int(row["movie_id"]), "title": row["title"]}}
            return {"available": True, "movie": None}
        except Exception:
            return {"available": False, "movie": None}

    return app


app = create_app()
