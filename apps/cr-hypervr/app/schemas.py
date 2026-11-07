from __future__ import annotations

from pydantic import BaseModel, Field
from typing import List, Optional


class EmbedTextRequest(BaseModel):
    text: str


class EmbedBatchRequest(BaseModel):
    texts: List[str]


class EmbedVectorResponse(BaseModel):
    embedding: List[float] = Field(..., description="384-dimensional embedding")
    dimension: int = Field(384, description="Embedding dimension")
    model: str = Field("movie-minilm-v1", description="Model identifier")


class SimilarSearchRequest(BaseModel):
    text: str
    top_k: int = 10


class SimilarItem(BaseModel):
    movie_id: int
    title: str
    genres: Optional[str] = None
    score: float


class SimilarSearchResponse(BaseModel):
    items: List[SimilarItem]


class RecommendRequest(BaseModel):
    user_id: int
    top_k: int = 10
    exclude_movie_ids: List[int] | None = None


class RecommendResponse(BaseModel):
    items: List[SimilarItem]


class MovieEmbedRequest(BaseModel):
    title: str
    description: Optional[str] = None
    genres: List[str] = Field(default_factory=list)


class UserEmbedRequest(BaseModel):
    liked_genres: List[str] = Field(default_factory=list)
    liked_movies: List[str] = Field(default_factory=list)
    disliked_genres: List[str] = Field(default_factory=list)


# --- Hypergraph query/recommendation ---
class GraphRecommendRequest(BaseModel):
    """Free‑text query that seeds vector search, then expands via hypergraph.

    Fields allow light tuning without overcomplicating the interface.
    """
    query: str = Field(..., description="User query to embed and seed the graph search")
    top_k: int = Field(10, description="Number of recommendations to return")
    seed_top_k: int = Field(20, description="Seed candidates from vector search")
    hops: int = Field(2, description="Depth for graph expansion (1=cowatch, 2=+genres)")
    embed_weight: float = Field(1.0, description="Weight for base embedding similarity")
    cowatch_weight: float = Field(0.5, description="Weight for co‑watch edges")
    genre_weight: float = Field(0.25, description="Weight for shared‑genre signal")


class GraphRecommendItem(SimilarItem):
    sources: list[str] | None = Field(default=None, description="Signals that contributed (embed|cowatch|genre)")


class GraphRecommendResponse(BaseModel):
    items: List[GraphRecommendItem]
