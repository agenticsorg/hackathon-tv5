FROM python:3.11-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1

WORKDIR /app

# System deps (minimal)
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*

COPY requirements.txt ./
RUN pip install --upgrade pip && pip install -r requirements.txt

# Preload base model for offline CPU inference (no heredoc for wider builder support)
RUN python -c "from sentence_transformers import SentenceTransformer; import os; os.makedirs('models/base-minilm', exist_ok=True); SentenceTransformer('sentence-transformers/all-MiniLM-L6-v2').save('models/base-minilm'); print('Base MiniLM model cached at models/base-minilm')"

COPY app ./app
COPY scripts ./scripts
COPY pipeline ./pipeline
COPY training ./training

EXPOSE 8080

ENV MODEL_DIR=""
ENV BASE_MODEL_DIR="models/base-minilm"
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8080"]
