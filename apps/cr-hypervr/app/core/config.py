from pydantic import Field
from pydantic_settings import BaseSettings
from typing import List


class Settings(BaseSettings):
    app_name: str = Field(default="CR-HyperVR")
    environment: str = Field(default="dev")
    log_level: str = Field(default="INFO")
    database_url: str | None = None
    model_dir: str = Field(default="models/movie-minilm-v1")
    base_model_dir: str = Field(default="models/base-minilm")
    model_name: str = Field(default="movie-minilm-v1")
    vector_dim: int = Field(default=384)
    allowed_origins: List[str] = Field(default_factory=list, description="CORS allowed origins")
    # Embedding backend: auto|onnx|st|hash (hash = deterministic lightweight backend for tests/offline)
    embedding_backend: str = Field(default="auto")
    # Graph scoring options
    use_graph_scorer: bool = Field(default=False)
    graph_score_weight: float = Field(default=0.05)
    # Optional reranker toggle (stub implementation)
    use_reranker: bool = Field(default=False)

    class Config:
        env_file = ".env"


settings = Settings()
