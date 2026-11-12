from __future__ import annotations

import threading
from functools import lru_cache
import os
import numpy as np

try:  # optional
    import onnxruntime as ort  # type: ignore
except Exception:  # pragma: no cover - optional
    ort = None

from app.core.config import settings


class _Embedder:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._mode = "st"
        self._st_model = None  # lazy import to avoid heavy deps during tests
        self._ort_session = None
        self._prepare()

    def _prepare(self) -> None:
        # Optional: pull fine-tuned model artifacts from GCS if requested
        try:
            self._maybe_pull_model_from_gcs()
        except Exception:
            # Best-effort; proceed with normal backend selection
            pass
        # Allow explicit backend override via env or settings
        override = os.getenv("EMBEDDING_BACKEND") or getattr(settings, "embedding_backend", "auto")
        if override == "hash":
            self._mode = "hash"
            return
        if override == "onnx":
            onnx_path = f"{settings.model_dir}/model-int8.onnx"
            if ort is not None:
                try:
                    self._ort_session = ort.InferenceSession(onnx_path, providers=["CPUExecutionProvider"])  # type: ignore[arg-type]
                    self._mode = "onnx"
                    return
                except Exception:
                    pass
            # fallback chain: ST → hash
            try:
                self._ensure_st()
                self._mode = "st"
                return
            except Exception:
                self._mode = "hash"
                return
        # Prefer ONNX if available (auto)
        onnx_path = f"{settings.model_dir}/model-int8.onnx"
        if ort is not None:
            try:
                self._ort_session = ort.InferenceSession(onnx_path, providers=["CPUExecutionProvider"])  # type: ignore[arg-type]
                self._mode = "onnx"
                return
            except Exception:
                pass
        # Fallback chain: ST → hash
        try:
            self._ensure_st()
            self._mode = "st"
            return
        except Exception:
            self._mode = "hash"
            return

    def _ensure_st(self) -> None:
        if self._st_model is None:
            from sentence_transformers import SentenceTransformer  # type: ignore
            self._st_model = SentenceTransformer(settings.model_dir if settings.model_dir else settings.base_model_dir)

    def _maybe_pull_model_from_gcs(self) -> None:
        # If a GCS URI is provided and local ONNX not present, pull down.
        gcs_uri = os.getenv("MODEL_GCS_URI") or os.getenv("GCS_MODEL_URI")
        if not gcs_uri or not gcs_uri.startswith("gs://"):
            # Derive from bucket hint if available
            bucket = os.getenv("GCS_MODELS_BUCKET")
            if bucket and bucket.startswith("gs://"):
                gcs_uri = f"{bucket.rstrip('/')}/models/movie-minilm-v1/model-int8.onnx"
            else:
                return
        # Destination
        model_dir = getattr(settings, "model_dir", "models/movie-minilm-v1") or "models/movie-minilm-v1"
        onnx_path = os.path.join(model_dir, "model-int8.onnx")
        # Already present → nothing to do
        if os.path.exists(onnx_path):
            return
        os.makedirs(model_dir, exist_ok=True)
        # Pull file or directory
        import fsspec

        fs = fsspec.filesystem("gcs")
        if gcs_uri.endswith(".onnx"):
            with fs.open(gcs_uri, "rb") as src, open(onnx_path, "wb") as dst:  # type: ignore[attr-defined]
                dst.write(src.read())
            return
        # Otherwise, treat as prefix and sync all files
        # Ensure trailing slash for globbing
        prefix = gcs_uri.rstrip("/") + "/"
        files = [p for p in fs.glob(prefix + "**") if not p.endswith("/")]
        for obj in files:
            rel = obj[len(prefix) :]
            local = os.path.join(model_dir, rel)
            os.makedirs(os.path.dirname(local), exist_ok=True)
            with fs.open(obj, "rb") as src, open(local, "wb") as dst:  # type: ignore[attr-defined]
                dst.write(src.read())

    def _encode_hash(self, texts: list[str]) -> np.ndarray:
        dim = int(getattr(settings, "vector_dim", 384))
        out = []
        for t in texts:
            seed = int.from_bytes(np.frombuffer(t.encode("utf-8"), dtype=np.uint8).sum().tobytes(), "little", signed=False) ^ (len(t) * 1315423911)
            rng = np.random.default_rng(seed)
            v = rng.standard_normal(dim)
            n = np.linalg.norm(v)
            if n == 0:
                out.append(np.zeros(dim, dtype=np.float32))
            else:
                out.append((v / n).astype(np.float32))
        return np.stack(out, axis=0)

    def encode(self, texts: list[str]) -> np.ndarray:
        with self._lock:
            if self._mode == "hash":
                return self._encode_hash(texts)
            if self._mode == "onnx" and self._ort_session is not None:
                # For simplicity, use ST encode even when ONNX present (pipeline optimizable later)
                self._ensure_st()
                vecs = self._st_model.encode(texts, normalize_embeddings=True, convert_to_numpy=True)
            else:
                self._ensure_st()
                vecs = self._st_model.encode(texts, normalize_embeddings=True, convert_to_numpy=True)
        return vecs.astype(np.float32)


@lru_cache(maxsize=1)
def get_embedder() -> _Embedder:
    return _Embedder()
