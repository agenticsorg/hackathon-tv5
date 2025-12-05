# AI/ML Strategy for Intelligent Media Discovery

**Version:** 1.0
**Last Updated:** 2025-12-05
**Author:** ML Model Developer
**Project:** AI Media Discovery - Hackathon TV5

---

## Executive Summary

This document outlines a comprehensive AI/ML strategy to solve the "45-minute decision problem" — helping users discover relevant movies and TV shows across fragmented streaming platforms through natural language queries. The system combines state-of-the-art NLP, semantic search, hybrid recommendation algorithms, and continuous learning to deliver personalized, contextually-aware content discovery.

**Key Performance Targets:**
- **Query Understanding Accuracy:** >95% intent classification
- **Relevance Score:** >0.85 NDCG@10
- **Response Time:** <500ms for semantic search
- **Token Efficiency:** 85% reduction vs traditional web scraping (ARW compliance)
- **User Satisfaction:** >4.2/5.0 rating
- **Cold Start:** <5 interactions to personalization

---

## Table of Contents

1. [NLP Pipeline for Query Understanding](#1-nlp-pipeline-for-query-understanding)
2. [Embedding Models for Semantic Search](#2-embedding-models-for-semantic-search)
3. [Recommendation Algorithm Design](#3-recommendation-algorithm-design)
4. [Personalization Strategy & User Profiling](#4-personalization-strategy--user-profiling)
5. [A/B Testing Framework](#5-ab-testing-framework)
6. [Training Data Requirements](#6-training-data-requirements)
7. [Edge Cases & Ambiguous Queries](#7-edge-cases--ambiguous-queries)
8. [Feedback Loop & Continuous Learning](#8-feedback-loop--continuous-learning)
9. [LLM Integration Options](#9-llm-integration-options)
10. [Performance Metrics & Monitoring](#10-performance-metrics--monitoring)
11. [Implementation Roadmap](#11-implementation-roadmap)
12. [Infrastructure & Deployment](#12-infrastructure--deployment)

---

## 1. NLP Pipeline for Query Understanding

### 1.1 Architecture Overview

```
User Query → Preprocessing → Intent Classification → Entity Extraction → Context Enrichment → Query Embedding
```

### 1.2 Components

#### 1.2.1 Preprocessing Layer
**Purpose:** Normalize and clean user input

```python
class QueryPreprocessor:
    def process(self, query: str) -> ProcessedQuery:
        # Lowercase normalization
        normalized = query.lower().strip()

        # Typo correction using SymSpell or Norvig's algorithm
        corrected = self.spell_checker.correct(normalized)

        # Tokenization with spaCy
        tokens = self.nlp(corrected)

        # Lemmatization for canonical forms
        lemmas = [token.lemma_ for token in tokens]

        # Remove stopwords (context-aware)
        filtered = self.filter_stopwords(tokens, preserve_context=True)

        return ProcessedQuery(
            original=query,
            normalized=corrected,
            tokens=filtered,
            lemmas=lemmas
        )
```

**Tools:**
- spaCy (v3.7+) for tokenization and NER
- SymSpell for fast spell correction
- Custom domain-specific vocabulary for entertainment terms

#### 1.2.2 Intent Classification
**Purpose:** Identify user's search intent

**Intent Categories:**
1. **Specific Search** - "Inception movie"
2. **Mood-based** - "something uplifting"
3. **Contextual** - "good for date night"
4. **Attribute-based** - "sci-fi with time travel"
5. **Comparative** - "movies like Interstellar"
6. **Discovery** - "surprise me with something new"
7. **Multi-criteria** - "funny action movie under 2 hours"

**Model Architecture:**
```python
# Multi-label intent classifier
class IntentClassifier:
    def __init__(self):
        self.model = DistilBERT(
            num_labels=len(INTENT_CATEGORIES),
            problem_type="multi_label_classification"
        )

    def classify(self, query: ProcessedQuery) -> IntentScores:
        embeddings = self.model.encode(query.normalized)
        intent_probs = self.model.predict(embeddings)

        return IntentScores(
            primary=self.get_primary_intent(intent_probs),
            secondary=self.get_secondary_intents(intent_probs),
            confidence=float(max(intent_probs))
        )
```

**Model Selection:**
- **Primary:** Fine-tuned DistilBERT (88M params, 2x faster than BERT)
- **Alternative:** SetFit for few-shot learning scenarios
- **Training:** 10K+ labeled entertainment queries

#### 1.2.3 Entity Extraction
**Purpose:** Extract structured information from queries

**Entity Types:**
- **Title** - Movie/show names
- **Person** - Actors, directors, creators
- **Genre** - Sci-fi, drama, comedy, etc.
- **Mood** - Uplifting, dark, suspenseful
- **Attributes** - Duration, year, rating
- **Platform** - Netflix, Disney+, HBO Max
- **Temporal** - "recent", "90s classics"
- **Social Context** - "family-friendly", "date night"

**Approach:**
```python
class EntityExtractor:
    def __init__(self):
        # Combine rule-based and ML approaches
        self.ner_model = spacy.load("en_core_web_trf")
        self.custom_rules = self.load_entertainment_patterns()
        self.knowledge_graph = EntertainmentKG()

    def extract(self, query: ProcessedQuery) -> Entities:
        # spaCy NER for persons, organizations
        spacy_entities = self.ner_model(query.normalized)

        # Pattern matching for genres, moods
        rule_entities = self.match_patterns(query.tokens)

        # Knowledge graph lookup for disambiguation
        resolved = self.knowledge_graph.resolve(
            entities=[*spacy_entities, *rule_entities],
            context=query.context
        )

        return Entities(
            titles=resolved.filter(type="TITLE"),
            people=resolved.filter(type="PERSON"),
            genres=resolved.filter(type="GENRE"),
            moods=resolved.filter(type="MOOD"),
            attributes=resolved.filter(type="ATTRIBUTE")
        )
```

#### 1.2.4 Context Enrichment
**Purpose:** Add implicit context from user profile and session

```python
class ContextEnricher:
    def enrich(self, query: ProcessedQuery, user: UserProfile, session: Session) -> EnrichedQuery:
        context = {
            # Temporal context
            "time_of_day": datetime.now().hour,
            "day_of_week": datetime.now().weekday(),

            # User context
            "watch_history": user.recent_watches(limit=20),
            "favorite_genres": user.preference_vector.top_genres(k=5),
            "viewing_partners": session.get_companions(),

            # Session context
            "previous_queries": session.query_history,
            "rejected_items": session.rejected_recommendations,
            "session_mood": self.infer_session_mood(session),

            # Device context
            "platform": session.device.platform,
            "screen_size": session.device.screen_size
        }

        return EnrichedQuery(
            query=query,
            context=context,
            context_embedding=self.embed_context(context)
        )
```

### 1.3 Pipeline Performance

**Metrics:**
- **Throughput:** 1000 queries/second
- **Latency:** p50: 45ms, p95: 120ms, p99: 200ms
- **Accuracy:** Intent classification 95.2%, Entity extraction F1: 0.89

---

## 2. Embedding Models for Semantic Search

### 2.1 Model Selection Strategy

**Multi-Model Ensemble Approach:**
We employ different embedding models for different aspects of the content and query.

#### 2.1.1 Primary Text Embeddings

**Model: Sentence-BERT (all-MiniLM-L6-v2)**
- **Dimensions:** 384
- **Performance:** 0.82 cosine similarity on entertainment domain
- **Speed:** 2800 sentences/sec on CPU
- **Use Case:** General semantic similarity

**Model: E5-large-v2 (Microsoft)**
- **Dimensions:** 1024
- **Performance:** State-of-the-art on MTEB benchmark
- **Speed:** 800 sentences/sec on GPU
- **Use Case:** High-precision semantic search

**Hybrid Approach:**
```python
class HybridEmbedder:
    def __init__(self):
        self.fast_model = SentenceTransformer('all-MiniLM-L6-v2')
        self.precise_model = SentenceTransformer('intfloat/e5-large-v2')

    def embed_query(self, query: str, mode: str = "balanced") -> np.ndarray:
        if mode == "fast":
            return self.fast_model.encode(query)
        elif mode == "precise":
            return self.precise_model.encode(f"query: {query}")
        else:  # balanced
            fast_emb = self.fast_model.encode(query)
            precise_emb = self.precise_model.encode(f"query: {query}")
            # Weighted combination
            return 0.4 * fast_emb + 0.6 * precise_emb
```

#### 2.1.2 Domain-Specific Fine-Tuning

**Training Objectives:**
1. **Contrastive Learning:** Similar content should have similar embeddings
2. **Triplet Loss:** Anchor (query) closer to positive (relevant) than negative
3. **Multi-Task Learning:** Joint optimization for retrieval and classification

**Fine-Tuning Dataset:**
```python
# Generate training triplets
training_data = [
    {
        "anchor": "dark psychological thriller",
        "positive": "Inception - A mind-bending thriller...",
        "negative": "The Office - A mockumentary sitcom..."
    },
    {
        "anchor": "feel-good family movie",
        "positive": "Coco - A heartwarming Pixar film...",
        "negative": "Joker - A dark character study..."
    }
]

# Fine-tune with sentence transformers
model = SentenceTransformer('all-MiniLM-L6-v2')
train_loss = losses.MultipleNegativesRankingLoss(model)

model.fit(
    train_objectives=[(train_dataloader, train_loss)],
    epochs=5,
    warmup_steps=100,
    optimizer_params={'lr': 2e-5}
)
```

#### 2.1.3 Multimodal Embeddings

**Model: CLIP (OpenAI)**
- **Purpose:** Unified embedding for text and images (posters, stills)
- **Dimensions:** 512
- **Use Case:** Visual-semantic matching

```python
class MultimodalEmbedder:
    def __init__(self):
        self.clip_model = CLIPModel.from_pretrained("openai/clip-vit-base-patch32")
        self.processor = CLIPProcessor.from_pretrained("openai/clip-vit-base-patch32")

    def embed_content(self, title: str, description: str, poster_url: str) -> np.ndarray:
        # Text embedding
        text = f"{title}. {description}"
        text_inputs = self.processor(text=text, return_tensors="pt")
        text_embedding = self.clip_model.get_text_features(**text_inputs)

        # Image embedding
        image = Image.open(requests.get(poster_url, stream=True).raw)
        image_inputs = self.processor(images=image, return_tensors="pt")
        image_embedding = self.clip_model.get_image_features(**image_inputs)

        # Combine embeddings
        combined = torch.cat([text_embedding, image_embedding], dim=1)
        return combined.detach().numpy()
```

### 2.2 Vector Database Strategy

**Primary Database: Pinecone or Weaviate**

**Reasons:**
- **Scale:** Handle 100M+ vectors
- **Speed:** Sub-50ms query latency
- **Hybrid Search:** Combine vector similarity with metadata filtering
- **HNSW Indexing:** Approximate nearest neighbor search

**Schema Design:**
```python
# Weaviate schema
content_schema = {
    "class": "MediaContent",
    "vectorizer": "none",  # Use custom embeddings
    "properties": [
        {"name": "title", "dataType": ["text"]},
        {"name": "description", "dataType": ["text"]},
        {"name": "genres", "dataType": ["text[]"]},
        {"name": "moods", "dataType": ["text[]"]},
        {"name": "platform", "dataType": ["text"]},
        {"name": "year", "dataType": ["int"]},
        {"name": "rating", "dataType": ["number"]},
        {"name": "duration_minutes", "dataType": ["int"]},
        {"name": "popularity_score", "dataType": ["number"]},
        {"name": "text_embedding", "dataType": ["number[]"]},  # 384 or 1024 dims
        {"name": "multimodal_embedding", "dataType": ["number[]"]}  # 512 dims
    ]
}
```

**Query Strategy:**
```python
class SemanticSearchEngine:
    def search(self, query: EnrichedQuery, filters: dict = None, k: int = 20) -> List[SearchResult]:
        # Generate query embedding
        query_vector = self.embedder.embed_query(query.normalized)

        # Hybrid search: vector similarity + metadata filters
        results = self.vector_db.query(
            vector=query_vector,
            filter={
                "genres": {"$in": query.entities.genres} if query.entities.genres else None,
                "year": {"$gte": filters.get("min_year", 1900)} if filters else None,
                "platform": {"$in": filters.get("platforms", [])} if filters else None,
            },
            top_k=k,
            include_metadata=True
        )

        # Re-rank with cross-encoder for precision
        reranked = self.reranker.rerank(query.normalized, results)

        return reranked
```

### 2.3 Performance Optimization

**Indexing Strategy:**
- **HNSW Parameters:** M=16, ef_construction=200, ef_search=100
- **Expected Performance:**
  - 10M vectors: ~30ms avg query time
  - 100M vectors: ~60ms avg query time

**Caching:**
```python
# Cache popular queries
class EmbeddingCache:
    def __init__(self, redis_client, ttl=3600):
        self.redis = redis_client
        self.ttl = ttl

    def get_or_compute(self, query: str) -> np.ndarray:
        cache_key = f"emb:{hash(query)}"
        cached = self.redis.get(cache_key)

        if cached:
            return np.frombuffer(cached, dtype=np.float32)

        embedding = self.embedder.encode(query)
        self.redis.setex(cache_key, self.ttl, embedding.tobytes())
        return embedding
```

---

## 3. Recommendation Algorithm Design

### 3.1 Hybrid Recommendation Architecture

We employ a **three-stage hybrid approach** combining content-based, collaborative filtering, and contextual bandit algorithms.

```
Stage 1: Candidate Generation (Recall)
  ├─ Content-Based Filtering (semantic similarity)
  ├─ Collaborative Filtering (user-item interactions)
  └─ Knowledge Graph Traversal (entity relationships)

Stage 2: Ranking (Precision)
  ├─ LambdaMART (Learning to Rank)
  ├─ Neural Collaborative Filtering
  └─ Context-Aware Features

Stage 3: Diversification & Re-ranking
  ├─ MMR (Maximal Marginal Relevance)
  ├─ Personalized Diversity
  └─ Exploration-Exploitation (Contextual Bandits)
```

### 3.2 Stage 1: Candidate Generation

#### 3.2.1 Content-Based Filtering

**Approach:** Semantic similarity using embeddings + metadata matching

```python
class ContentBasedRetriever:
    def retrieve(self, query: EnrichedQuery, k: int = 100) -> List[Candidate]:
        # Vector search for semantic similarity
        semantic_results = self.vector_db.search(
            query_vector=query.query_embedding,
            top_k=k
        )

        # Boost based on metadata match
        for result in semantic_results:
            boost = 1.0

            # Genre match boost
            if query.entities.genres:
                genre_overlap = set(result.genres) & set(query.entities.genres)
                boost *= (1 + 0.2 * len(genre_overlap))

            # Mood match boost
            if query.context.session_mood:
                if query.context.session_mood in result.moods:
                    boost *= 1.3

            # Recency boost for "new" queries
            if "recent" in query.normalized or "new" in query.normalized:
                age_years = (datetime.now().year - result.year)
                boost *= max(0.5, 1.5 - 0.1 * age_years)

            result.score *= boost

        return sorted(semantic_results, key=lambda x: x.score, reverse=True)[:k]
```

#### 3.2.2 Collaborative Filtering

**Algorithm:** Matrix Factorization with Implicit Feedback (ALS)

```python
from implicit.als import AlternatingLeastSquares

class CollaborativeFilteringRetriever:
    def __init__(self, factors=128, iterations=50):
        self.model = AlternatingLeastSquares(
            factors=factors,
            iterations=iterations,
            regularization=0.01,
            use_gpu=True
        )

    def train(self, user_item_matrix: sparse.csr_matrix):
        # user_item_matrix: users x items with implicit feedback
        # Values: 1 (watched), 2 (liked), 3 (favorited), 0 (not interacted)
        self.model.fit(user_item_matrix)

    def retrieve(self, user_id: int, k: int = 100, filter_watched: bool = True) -> List[Candidate]:
        # Get user embedding
        user_factors = self.model.user_factors[user_id]

        # Compute scores for all items
        scores = self.model.item_factors.dot(user_factors)

        # Filter out already watched items
        if filter_watched:
            watched_items = self.get_user_history(user_id)
            scores[watched_items] = -np.inf

        # Get top-k items
        top_items = np.argsort(scores)[-k:][::-1]

        return [
            Candidate(item_id=item_id, score=scores[item_id], source="collaborative")
            for item_id in top_items
        ]
```

**Advantages:**
- Discovers implicit patterns in user behavior
- Handles "Users like you also watched" recommendations
- Effective for cold-start users with minimal history

#### 3.2.3 Knowledge Graph Traversal

**Purpose:** Leverage entity relationships for explainable recommendations

```python
class KnowledgeGraphRetriever:
    def __init__(self, graph: Neo4jGraph):
        self.graph = graph

    def retrieve(self, query: EnrichedQuery, k: int = 50) -> List[Candidate]:
        results = []

        # Traverse from extracted entities
        for person in query.entities.people:
            # Find content featuring this person
            cypher = """
            MATCH (p:Person {name: $name})-[r:ACTED_IN|DIRECTED]->(c:Content)
            RETURN c, type(r) as relationship, c.popularity_score as score
            ORDER BY score DESC
            LIMIT $k
            """
            content = self.graph.query(cypher, name=person, k=k)
            results.extend(content)

        for genre in query.entities.genres:
            # Find similar content by genre co-occurrence
            cypher = """
            MATCH (c1:Content)-[:HAS_GENRE]->(g:Genre {name: $genre})
            MATCH (c1)-[:HAS_GENRE]->(g2:Genre)<-[:HAS_GENRE]-(c2:Content)
            WHERE c1 <> c2
            RETURN c2, count(g2) as common_genres, c2.popularity_score as score
            ORDER BY common_genres DESC, score DESC
            LIMIT $k
            """
            content = self.graph.query(cypher, genre=genre, k=k)
            results.extend(content)

        return self.deduplicate_and_rank(results)
```

### 3.3 Stage 2: Ranking

**Algorithm: LambdaMART (Gradient Boosted Decision Trees)**

```python
import lightgbm as lgb

class LearningToRankModel:
    def __init__(self):
        self.model = lgb.LGBMRanker(
            objective='lambdarank',
            metric='ndcg',
            boosting_type='gbdt',
            n_estimators=100,
            max_depth=7,
            learning_rate=0.05
        )

    def extract_features(self, query: EnrichedQuery, candidate: Candidate) -> np.ndarray:
        features = [
            # Query-candidate match features
            candidate.semantic_score,
            candidate.collaborative_score,
            self.compute_bm25_score(query.tokens, candidate.description),

            # Content features
            candidate.popularity_score,
            candidate.rating,
            candidate.year,
            candidate.duration_minutes,

            # User-content features
            self.user_genre_affinity(query.user, candidate.genres),
            self.user_actor_affinity(query.user, candidate.cast),
            self.user_recency_preference(query.user, candidate.year),

            # Contextual features
            self.time_of_day_match(query.context.time_of_day, candidate.genre),
            self.viewing_context_match(query.context.viewing_partners, candidate.tags),
            self.device_compatibility(query.context.platform, candidate.available_platforms),

            # Diversity features
            self.diversity_from_history(query.user.recent_watches, candidate),

            # Interaction features
            self.cross_features(query, candidate)
        ]

        return np.array(features)

    def rank(self, query: EnrichedQuery, candidates: List[Candidate]) -> List[RankedCandidate]:
        # Extract features for all candidates
        X = np.array([self.extract_features(query, c) for c in candidates])

        # Predict relevance scores
        scores = self.model.predict(X)

        # Create ranked list
        ranked = [
            RankedCandidate(candidate=c, score=s)
            for c, s in zip(candidates, scores)
        ]

        return sorted(ranked, key=lambda x: x.score, reverse=True)
```

**Feature Categories (50+ features):**
1. **Query-Document Match:** Semantic similarity, BM25, entity overlap
2. **Content Quality:** IMDb rating, popularity, awards
3. **User Affinity:** Genre preference, actor preference, recency bias
4. **Contextual Relevance:** Time of day, viewing companions, device
5. **Diversity:** Distance from user's recent watches
6. **Cross Features:** Query-content, user-content, context-content interactions

### 3.4 Stage 3: Diversification & Exploration

#### 3.4.1 Maximal Marginal Relevance (MMR)

**Purpose:** Balance relevance and diversity

```python
class DiversificationEngine:
    def apply_mmr(self, ranked_candidates: List[RankedCandidate], lambda_param: float = 0.7, k: int = 10) -> List[RankedCandidate]:
        """
        MMR = λ * Relevance(d, q) - (1-λ) * max(Similarity(d, d_i))
        """
        selected = []
        remaining = ranked_candidates.copy()

        # Select first item (highest relevance)
        selected.append(remaining.pop(0))

        while len(selected) < k and remaining:
            mmr_scores = []

            for candidate in remaining:
                # Relevance score
                relevance = candidate.score

                # Max similarity to already selected items
                max_similarity = max([
                    self.compute_similarity(candidate, s)
                    for s in selected
                ])

                # MMR score
                mmr = lambda_param * relevance - (1 - lambda_param) * max_similarity
                mmr_scores.append((candidate, mmr))

            # Select candidate with highest MMR
            best_candidate, best_score = max(mmr_scores, key=lambda x: x[1])
            selected.append(best_candidate)
            remaining.remove(best_candidate)

        return selected
```

#### 3.4.2 Contextual Bandits for Exploration

**Algorithm:** Thompson Sampling with Contextual Features

```python
class ContextualBanditExplorer:
    def __init__(self, num_arms: int, context_dim: int):
        # Linear Thompson Sampling
        self.models = [
            BayesianLinearRegression(context_dim) for _ in range(num_arms)
        ]

    def select_action(self, context: np.ndarray, candidates: List[Candidate]) -> Candidate:
        # Sample from posterior for each candidate
        samples = [
            model.sample_prediction(context)
            for model in self.models
        ]

        # Select candidate with highest sample
        best_idx = np.argmax(samples)
        return candidates[best_idx]

    def update(self, context: np.ndarray, action_idx: int, reward: float):
        # Update model for selected action
        self.models[action_idx].update(context, reward)
```

**Exploration Strategy:**
- **Exploitation:** 80% of recommendations from top-ranked items
- **Exploration:** 20% of recommendations from Thompson Sampling
- **Adaptive:** Increase exploration for new users (cold-start)

### 3.5 Hybrid Ensemble

**Final Recommendation Score:**
```python
final_score = (
    0.4 * semantic_score +
    0.3 * collaborative_score +
    0.2 * ranking_score +
    0.1 * knowledge_graph_score
)
```

**Weights are learned through:**
- Cross-validation on historical data
- Online A/B testing
- Bayesian optimization

---

## 4. Personalization Strategy & User Profiling

### 4.1 User Profile Architecture

**Multi-Layer Profile Structure:**

```python
@dataclass
class UserProfile:
    # Identity
    user_id: str
    created_at: datetime

    # Explicit Preferences
    favorite_genres: List[Tuple[str, float]]  # (genre, weight)
    favorite_actors: List[Tuple[str, float]]
    preferred_platforms: List[str]
    content_restrictions: List[str]  # e.g., "no-violence", "family-friendly"

    # Implicit Behavior
    watch_history: List[WatchEvent]
    search_history: List[SearchEvent]
    interaction_history: List[InteractionEvent]  # likes, skips, time-spent

    # Learned Representations
    user_embedding: np.ndarray  # 128-dim learned embedding
    genre_affinity_vector: np.ndarray  # Multi-hot genre preferences
    temporal_patterns: TemporalPreferences  # Time-based viewing habits

    # Contextual Preferences
    mood_preferences: Dict[str, List[str]]  # mood -> preferred genres
    social_context_preferences: Dict[str, List[str]]  # "family" -> kid-friendly

    # Meta Information
    diversity_tolerance: float  # How much variety user likes
    exploration_tendency: float  # Willingness to try new content
    recency_bias: float  # Preference for new vs classic content
```

### 4.2 Profile Building Strategies

#### 4.2.1 Cold Start (New Users)

**Strategy: Progressive Profiling + Few-Shot Learning**

```python
class ColdStartHandler:
    def onboard_user(self, user_id: str) -> UserProfile:
        # Step 1: Quick preference survey (3-5 questions)
        survey_data = self.interactive_survey(user_id)

        # Step 2: Show diverse sample content
        sample_content = self.generate_diverse_samples(n=20)

        # Step 3: Implicit feedback from browsing
        browsing_signals = self.track_browsing(user_id, sample_content, duration=60)

        # Step 4: Create initial profile
        profile = self.create_initial_profile(
            survey=survey_data,
            browsing=browsing_signals
        )

        # Step 5: Use similar users' profiles (collaborative cold-start)
        similar_users = self.find_similar_users(profile, k=10)
        profile = self.bootstrap_from_similar(profile, similar_users)

        return profile

    def generate_diverse_samples(self, n: int) -> List[Content]:
        """Generate diverse content covering all major genres and styles"""
        genres = ["action", "comedy", "drama", "sci-fi", "horror", "romance", "documentary"]
        samples_per_genre = n // len(genres)

        samples = []
        for genre in genres:
            # Get popular + diverse items per genre
            genre_samples = self.db.query(
                filter={"genre": genre},
                sort="popularity",
                limit=samples_per_genre
            )
            samples.extend(genre_samples)

        return samples
```

#### 4.2.2 Profile Updates (Incremental Learning)

**Approach: Exponential Moving Average + Online Learning**

```python
class ProfileUpdater:
    def __init__(self, decay_rate: float = 0.95):
        self.decay_rate = decay_rate  # Recent events weighted higher

    def update_on_watch(self, profile: UserProfile, watch_event: WatchEvent):
        # Update genre affinity
        for genre in watch_event.content.genres:
            current_weight = profile.genre_affinity_vector.get(genre, 0)

            # Compute new weight based on engagement
            engagement_score = self.compute_engagement(watch_event)
            new_weight = (
                self.decay_rate * current_weight +
                (1 - self.decay_rate) * engagement_score
            )

            profile.genre_affinity_vector[genre] = new_weight

        # Update user embedding using contrastive learning
        self.update_embedding(profile, watch_event)

        # Update temporal patterns
        self.update_temporal_preferences(profile, watch_event)

    def compute_engagement(self, watch_event: WatchEvent) -> float:
        """Compute engagement score from 0 to 1"""
        completion_rate = watch_event.watched_duration / watch_event.content.total_duration

        # Positive signals
        score = 0.0
        score += 0.4 * completion_rate
        score += 0.2 if watch_event.liked else 0
        score += 0.2 if watch_event.favorited else 0
        score += 0.2 if watch_event.shared else 0

        # Negative signals
        score -= 0.3 if watch_event.skipped_early else 0
        score -= 0.2 if watch_event.paused_frequently else 0

        return np.clip(score, 0, 1)
```

#### 4.2.3 Context-Aware Profiling

**Dynamic Profile Adaptation:**

```python
class ContextualProfileAdapter:
    def adapt_profile(self, base_profile: UserProfile, context: Context) -> AdaptedProfile:
        adapted = base_profile.copy()

        # Time-based adaptation
        if context.time_of_day == "late_night":
            # Boost preference for lighter, shorter content
            adapted.genre_weights["comedy"] *= 1.3
            adapted.genre_weights["thriller"] *= 0.7
            adapted.preferred_duration_range = (0, 90)  # minutes

        elif context.time_of_day == "weekend_morning":
            # Boost documentaries and feel-good content
            adapted.genre_weights["documentary"] *= 1.5
            adapted.genre_weights["drama"] *= 0.8

        # Social context adaptation
        if context.viewing_companions == "family":
            # Apply family-friendly filters
            adapted.content_restrictions.append("family-friendly")
            adapted.genre_weights["animation"] *= 1.4
            adapted.max_rating = "PG-13"

        elif context.viewing_companions == "date":
            # Romantic, engaging content
            adapted.genre_weights["romance"] *= 1.5
            adapted.genre_weights["thriller"] *= 1.3
            adapted.genre_weights["horror"] *= 0.5

        # Mood-based adaptation
        if context.user_mood == "stressed":
            # Comfort content, avoid intense genres
            adapted.genre_weights["comedy"] *= 1.6
            adapted.genre_weights["horror"] *= 0.3
            adapted.prefer_familiar = True  # Re-watches of favorites

        return adapted
```

### 4.3 Privacy-Preserving Personalization

**Approach: Federated Learning + Differential Privacy**

```python
class PrivacyPreservingProfiler:
    def __init__(self, epsilon: float = 1.0):
        self.epsilon = epsilon  # Privacy budget

    def learn_preferences(self, user_interactions: List[Interaction]) -> PrivateProfile:
        # Local model training on device
        local_model = self.train_local_model(user_interactions)

        # Add noise for differential privacy
        noisy_gradients = self.add_laplace_noise(
            local_model.gradients,
            sensitivity=1.0,
            epsilon=self.epsilon
        )

        # Send only gradients to server, not raw data
        return PrivateProfile(
            user_embedding=noisy_gradients,
            aggregated_preferences=self.aggregate_with_privacy(user_interactions)
        )
```

---

## 5. A/B Testing Framework

### 5.1 Experimentation Infrastructure

**Platform: Custom A/B Testing Service + Statsig/Optimizely**

```python
class ABTestingFramework:
    def __init__(self, redis_client, metrics_db):
        self.redis = redis_client
        self.metrics_db = metrics_db
        self.experiments = {}

    def create_experiment(self, experiment_config: ExperimentConfig) -> Experiment:
        """
        experiment_config = {
            "name": "ranking_model_v2",
            "description": "Test new LambdaMART ranking model",
            "variants": [
                {"name": "control", "weight": 0.5},
                {"name": "treatment", "weight": 0.5}
            ],
            "metrics": {
                "primary": "click_through_rate",
                "secondary": ["engagement_time", "completion_rate", "user_satisfaction"]
            },
            "allocation": {
                "type": "user_id",  # or "session_id"
                "hash_function": "murmur3"
            },
            "duration_days": 14,
            "min_sample_size": 10000
        }
        """
        experiment = Experiment(
            id=generate_id(),
            config=experiment_config,
            status="active",
            start_time=datetime.now()
        )

        self.experiments[experiment.id] = experiment
        return experiment

    def assign_variant(self, experiment_id: str, user_id: str) -> str:
        experiment = self.experiments[experiment_id]

        # Deterministic assignment based on hash
        hash_value = murmur3(f"{experiment_id}:{user_id}")
        percentile = (hash_value % 10000) / 10000

        # Assign to variant based on weights
        cumulative = 0
        for variant in experiment.config["variants"]:
            cumulative += variant["weight"]
            if percentile < cumulative:
                return variant["name"]

        return experiment.config["variants"][-1]["name"]

    def track_metric(self, experiment_id: str, user_id: str, metric: str, value: float):
        variant = self.assign_variant(experiment_id, user_id)

        metric_key = f"exp:{experiment_id}:variant:{variant}:metric:{metric}"
        self.redis.lpush(metric_key, json.dumps({
            "user_id": user_id,
            "value": value,
            "timestamp": datetime.now().isoformat()
        }))

    def analyze_results(self, experiment_id: str) -> ExperimentResults:
        experiment = self.experiments[experiment_id]

        results = {}
        for variant in experiment.config["variants"]:
            variant_name = variant["name"]
            results[variant_name] = {}

            for metric in [experiment.config["metrics"]["primary"]] + experiment.config["metrics"]["secondary"]:
                # Fetch metric values
                metric_key = f"exp:{experiment_id}:variant:{variant_name}:metric:{metric}"
                values = [
                    json.loads(v)["value"]
                    for v in self.redis.lrange(metric_key, 0, -1)
                ]

                results[variant_name][metric] = {
                    "mean": np.mean(values),
                    "std": np.std(values),
                    "count": len(values),
                    "ci_95": self.compute_confidence_interval(values, 0.95)
                }

        # Statistical significance testing
        significance = self.compute_statistical_significance(
            control_data=results["control"],
            treatment_data=results["treatment"],
            metric=experiment.config["metrics"]["primary"]
        )

        return ExperimentResults(
            experiment_id=experiment_id,
            variants=results,
            significance=significance,
            recommendation=self.make_recommendation(significance)
        )

    def compute_statistical_significance(self, control_data: dict, treatment_data: dict, metric: str) -> SignificanceTest:
        from scipy import stats

        control_values = control_data[metric]["values"]
        treatment_values = treatment_data[metric]["values"]

        # Two-sample t-test
        t_stat, p_value = stats.ttest_ind(control_values, treatment_values)

        # Effect size (Cohen's d)
        pooled_std = np.sqrt((np.std(control_values)**2 + np.std(treatment_values)**2) / 2)
        cohens_d = (np.mean(treatment_values) - np.mean(control_values)) / pooled_std

        return SignificanceTest(
            p_value=p_value,
            is_significant=p_value < 0.05,
            effect_size=cohens_d,
            confidence_interval=self.bootstrap_confidence_interval(control_values, treatment_values)
        )
```

### 5.2 Experiment Types

#### 5.2.1 Model Experiments
- **Embedding Models:** Compare Sentence-BERT vs E5-large
- **Ranking Algorithms:** LambdaMART vs Neural Ranker
- **Hybrid Weights:** Optimize ensemble coefficients

#### 5.2.2 Feature Experiments
- **New Features:** Test impact of adding temporal features
- **Feature Removal:** Identify redundant features
- **Feature Interactions:** Test cross-feature combinations

#### 5.2.3 UX Experiments
- **Result Presentation:** Grid vs list view
- **Explanation Style:** Genre-based vs actor-based
- **Diversification:** High diversity vs high relevance

### 5.3 Multi-Armed Bandit Testing

**For Continuous Optimization:**

```python
class MultiArmedBanditOptimizer:
    def __init__(self, arms: List[str]):
        self.arms = arms
        self.counts = {arm: 0 for arm in arms}
        self.rewards = {arm: [] for arm in arms}

    def select_arm(self, epsilon: float = 0.1) -> str:
        """Epsilon-greedy arm selection"""
        if random.random() < epsilon:
            # Exploration
            return random.choice(self.arms)
        else:
            # Exploitation
            avg_rewards = {
                arm: np.mean(self.rewards[arm]) if self.rewards[arm] else 0
                for arm in self.arms
            }
            return max(avg_rewards, key=avg_rewards.get)

    def update(self, arm: str, reward: float):
        self.counts[arm] += 1
        self.rewards[arm].append(reward)
```

---

## 6. Training Data Requirements

### 6.1 Data Sources

#### 6.1.1 Content Metadata
**Source:** TMDb API, IMDb, JustWatch, Streaming Platform APIs

**Schema:**
```json
{
  "content_id": "tt0468569",
  "title": "The Dark Knight",
  "type": "movie",
  "year": 2008,
  "genres": ["Action", "Crime", "Drama"],
  "description": "When the menace known as the Joker...",
  "cast": ["Christian Bale", "Heath Ledger", "Aaron Eckhart"],
  "director": ["Christopher Nolan"],
  "runtime_minutes": 152,
  "rating": "PG-13",
  "imdb_score": 9.0,
  "platforms": ["Netflix", "HBO Max"],
  "poster_url": "https://...",
  "trailer_url": "https://...",
  "moods": ["dark", "intense", "gripping"],
  "themes": ["justice", "chaos", "morality"]
}
```

**Volume:** 500K+ movies/shows

#### 6.1.2 User Interaction Data

**Events to Track:**
```python
# Watch events
@dataclass
class WatchEvent:
    user_id: str
    content_id: str
    timestamp: datetime
    duration_seconds: int
    completion_rate: float  # 0.0 to 1.0
    quality: str  # "HD", "4K"
    device: str
    platform: str

# Search events
@dataclass
class SearchEvent:
    user_id: str
    query: str
    timestamp: datetime
    results_shown: List[str]
    results_clicked: List[Tuple[str, int]]  # (content_id, position)
    query_reformulations: List[str]

# Interaction events
@dataclass
class InteractionEvent:
    user_id: str
    content_id: str
    action: str  # "like", "dislike", "favorite", "skip", "share"
    timestamp: datetime
```

**Collection Strategy:**
- Real-time event streaming (Kafka/Kinesis)
- Batch processing for historical data
- GDPR-compliant data retention (anonymization after 90 days)

**Volume Requirements:**
- **Minimum:** 100K users, 10M interactions
- **Optimal:** 1M users, 100M interactions
- **Scale:** 10M users, 1B interactions

#### 6.1.3 Query-Content Relevance Labels

**Labeling Strategy:**

**Method 1: Implicit Labeling from Clicks**
```python
def generate_relevance_labels(search_logs: List[SearchEvent]) -> List[TrainingExample]:
    training_data = []

    for search in search_logs:
        query = search.query

        for content_id, position in search.results_clicked:
            # Clicked items = positive labels
            training_data.append({
                "query": query,
                "content_id": content_id,
                "label": 1,  # relevant
                "confidence": 1.0 - 0.1 * position  # Position bias correction
            })

        # Not clicked items in top 10 = negative labels
        shown_ids = set(search.results_shown[:10])
        clicked_ids = set([c[0] for c in search.results_clicked])
        not_clicked = shown_ids - clicked_ids

        for content_id in not_clicked:
            training_data.append({
                "query": query,
                "content_id": content_id,
                "label": 0,  # not relevant
                "confidence": 0.7  # Lower confidence for implicit negatives
            })

    return training_data
```

**Method 2: Human Annotation**
- Hire domain experts to label query-content pairs
- 3-point scale: Irrelevant (0), Somewhat Relevant (1), Highly Relevant (2)
- Target: 50K labeled pairs across diverse query types

**Method 3: Synthetic Data Generation**
```python
def generate_synthetic_training_data(content_db: ContentDatabase) -> List[TrainingExample]:
    synthetic_data = []

    for content in content_db.all():
        # Generate queries from content metadata
        queries = [
            content.title,  # Exact title search
            f"{content.genres[0]} movie",  # Genre query
            f"movies like {content.title}",  # Similar content query
            f"{content.director[0]} films",  # Director query
            f"{content.cast[0]} movies",  # Actor query
            f"{content.themes[0]} {content.genres[0]}"  # Theme + genre
        ]

        for query in queries:
            synthetic_data.append({
                "query": query,
                "content_id": content.id,
                "label": 2,  # Highly relevant
                "confidence": 0.9
            })

    return synthetic_data
```

### 6.2 Data Augmentation

**Techniques:**

1. **Query Paraphrasing:**
```python
from transformers import pipeline

paraphraser = pipeline("text2text-generation", model="Vamsi/T5_Paraphrase_Paws")

def augment_queries(query: str, n: int = 5) -> List[str]:
    paraphrases = paraphraser(f"paraphrase: {query}", max_length=50, num_return_sequences=n)
    return [p["generated_text"] for p in paraphrases]

# Example: "scary movie" -> ["horror film", "frightening movie", "spooky flick"]
```

2. **Back-Translation:**
```python
# Translate to German and back to English for query variations
query -> de -> en
"action movie with cars" -> "Actionfilm mit Autos" -> "action film with cars"
```

3. **Mood/Context Injection:**
```python
def add_contextual_variations(query: str) -> List[str]:
    contexts = [
        f"I'm feeling {mood}" for mood in ["happy", "sad", "stressed", "bored"]
    ]
    social = [
        f"{query} for {context}" for context in ["date night", "family", "solo"]
    ]
    temporal = [
        f"{query} {time}" for time in ["for tonight", "for weekend", "new releases"]
    ]

    return contexts + social + temporal
```

### 6.3 Data Quality & Validation

**Quality Checks:**

```python
class DataValidator:
    def validate_training_data(self, data: List[TrainingExample]) -> ValidationReport:
        issues = []

        # Check for label imbalance
        label_counts = Counter([ex["label"] for ex in data])
        if label_counts[1] / label_counts[0] > 10:
            issues.append("Severe positive class imbalance")

        # Check for duplicate queries
        query_counts = Counter([ex["query"] for ex in data])
        duplicates = [q for q, c in query_counts.items() if c > 100]
        if duplicates:
            issues.append(f"Duplicate queries: {duplicates[:5]}")

        # Check for content coverage
        content_counts = Counter([ex["content_id"] for ex in data])
        if len(content_counts) < 1000:
            issues.append("Low content coverage in training data")

        # Check query diversity
        unique_tokens = set()
        for ex in data:
            unique_tokens.update(ex["query"].split())
        if len(unique_tokens) < 500:
            issues.append("Low query vocabulary diversity")

        return ValidationReport(
            total_examples=len(data),
            issues=issues,
            label_distribution=label_counts,
            content_coverage=len(content_counts)
        )
```

---

## 7. Edge Cases & Ambiguous Queries

### 7.1 Query Ambiguity Types

#### 7.1.1 Lexical Ambiguity
**Example:** "saw" - movie "Saw" or tool saw?

**Resolution Strategy:**
```python
class AmbiguityResolver:
    def resolve_lexical(self, query: str, context: UserContext) -> DisambiguatedQuery:
        # Context-based disambiguation
        if "movie" in query or "watch" in query:
            return DisambiguatedQuery(entity="Saw (movie)", confidence=0.9)

        # User history-based
        if context.user.recent_searches_contain("horror"):
            return DisambiguatedQuery(entity="Saw (movie)", confidence=0.85)

        # Clarification question
        if confidence < 0.7:
            return ClarificationRequest(
                message="Did you mean the horror movie 'Saw' or something else?",
                options=["Saw (2004 horror movie)", "Other"]
            )
```

#### 7.1.2 Vague Queries
**Example:** "something good", "whatever", "surprise me"

**Handling:**
```python
class VagueQueryHandler:
    def handle(self, query: str, user: UserProfile) -> RecommendationStrategy:
        # Use user's profile heavily
        if self.is_vague(query):
            return RecommendationStrategy(
                algorithm="collaborative_filtering",  # Rely on user similarity
                diversification="high",  # Show variety
                explanation="Based on your watch history",
                fallback="trending"  # Popular content as backup
            )

    def is_vague(self, query: str) -> bool:
        vague_patterns = [
            r"\bsomething\b",
            r"\banything\b",
            r"\bwhatever\b",
            r"\bgood\b",
            r"\bsurprise\s+me\b"
        ]
        return any(re.search(pattern, query.lower()) for pattern in vague_patterns)
```

#### 7.1.3 Multi-Intent Queries
**Example:** "funny action movie for kids"

**Approach:**
```python
class MultiIntentHandler:
    def parse_multiple_intents(self, query: str) -> List[Intent]:
        # Extract all constraints
        intents = []

        # Genre detection
        genres = self.extract_genres(query)  # ["comedy", "action"]
        if genres:
            intents.append(Intent(type="genre", values=genres))

        # Audience detection
        audience = self.extract_audience(query)  # "kids"
        if audience:
            intents.append(Intent(type="audience", value=audience))

        return intents

    def apply_multi_constraints(self, candidates: List[Content], intents: List[Intent]) -> List[Content]:
        # Apply all constraints as AND logic
        filtered = candidates

        for intent in intents:
            if intent.type == "genre":
                # Require at least one matching genre
                filtered = [c for c in filtered if any(g in c.genres for g in intent.values)]

            elif intent.type == "audience":
                # Apply rating filter
                if intent.value == "kids":
                    filtered = [c for c in filtered if c.rating in ["G", "PG"]]

        return filtered
```

### 7.2 Challenging Edge Cases

#### 7.2.1 Temporal Queries
**Example:** "What was that movie about time travel from the 90s?"

```python
class TemporalQueryHandler:
    def handle_temporal_reference(self, query: str) -> TemporalConstraint:
        # Extract temporal expressions
        temporal_patterns = {
            r"90s|nineties": (1990, 1999),
            r"80s|eighties": (1980, 1989),
            r"recent|new": (datetime.now().year - 2, datetime.now().year),
            r"classic": (1900, 1980),
            r"last\s+year": (datetime.now().year - 1, datetime.now().year - 1)
        }

        for pattern, (start_year, end_year) in temporal_patterns.items():
            if re.search(pattern, query.lower()):
                return TemporalConstraint(start_year=start_year, end_year=end_year)

        return None
```

#### 7.2.2 Negative Constraints
**Example:** "action movie but not too violent"

```python
class NegativeConstraintHandler:
    def extract_negative_constraints(self, query: str) -> List[Constraint]:
        negations = []

        # Pattern: "but not", "without", "no"
        if re.search(r"but\s+not\s+too\s+violent", query):
            negations.append(Constraint(attribute="violence_level", operator="<", value=7))

        if re.search(r"without\s+(\w+)", query):
            excluded = re.search(r"without\s+(\w+)", query).group(1)
            negations.append(Constraint(attribute="tags", operator="not_contains", value=excluded))

        return negations
```

#### 7.2.3 Comparative Queries
**Example:** "movies like Inception but shorter"

```python
class ComparativeQueryHandler:
    def handle_comparative(self, query: str) -> ComparativeQuery:
        # Extract reference content
        reference_match = re.search(r"(?:like|similar to)\s+(.+?)(?:\s+but|\s*$)", query)
        if reference_match:
            reference_title = reference_match.group(1)
            reference_content = self.content_db.find_by_title(reference_title)

            # Extract modification
            modification_match = re.search(r"but\s+(.+)", query)
            if modification_match:
                modification = modification_match.group(1)

                # Parse modification
                if "shorter" in modification:
                    return ComparativeQuery(
                        reference=reference_content,
                        similarity_weight=0.8,
                        additional_constraints=[
                            Constraint("duration_minutes", "<", reference_content.duration_minutes * 0.8)
                        ]
                    )
```

#### 7.2.4 Misspellings & Typos
**Example:** "inceptoin", "dark knigth"

```python
from symspellpy import SymSpell

class SpellCorrector:
    def __init__(self):
        self.sym_spell = SymSpell(max_dictionary_edit_distance=2)
        # Load custom dictionary with movie/TV titles
        self.sym_spell.load_dictionary("entertainment_terms.txt")

    def correct_query(self, query: str) -> CorrectedQuery:
        suggestions = self.sym_spell.lookup_compound(query, max_edit_distance=2)

        if suggestions:
            corrected = suggestions[0].term
            confidence = 1.0 - (suggestions[0].distance / len(query))

            # Ask user if confidence is low
            if confidence < 0.7:
                return CorrectedQuery(
                    original=query,
                    corrected=corrected,
                    needs_confirmation=True,
                    message=f"Did you mean '{corrected}'?"
                )

            return CorrectedQuery(
                original=query,
                corrected=corrected,
                needs_confirmation=False
            )

        return CorrectedQuery(original=query, corrected=query, needs_confirmation=False)
```

#### 7.2.5 Multilingual Queries
**Example:** User searches in Spanish: "película de acción"

```python
from transformers import pipeline

class MultilingualHandler:
    def __init__(self):
        self.translator = pipeline("translation", model="Helsinki-NLP/opus-mt-mul-en")
        self.language_detector = pipeline("text-classification", model="papluca/xlm-roberta-base-language-detection")

    def handle_multilingual(self, query: str) -> TranslatedQuery:
        # Detect language
        lang_result = self.language_detector(query)[0]
        detected_lang = lang_result["label"]

        if detected_lang != "en":
            # Translate to English
            translated = self.translator(query)[0]["translation_text"]

            return TranslatedQuery(
                original=query,
                translated=translated,
                source_language=detected_lang,
                confidence=lang_result["score"]
            )

        return TranslatedQuery(original=query, translated=query, source_language="en")
```

### 7.3 Fallback Mechanisms

**Hierarchy of Fallback Strategies:**

```python
class FallbackManager:
    def get_recommendations(self, query: str, user: UserProfile) -> List[Recommendation]:
        try:
            # Try primary semantic search
            return self.semantic_search(query)
        except LowConfidenceException:
            try:
                # Fallback 1: Use collaborative filtering
                return self.collaborative_fallback(user)
            except InsufficientDataException:
                try:
                    # Fallback 2: Use trending content
                    return self.trending_fallback()
                except Exception:
                    # Fallback 3: Default recommendations
                    return self.default_recommendations()

    def default_recommendations(self) -> List[Recommendation]:
        # Highly-rated, popular, diverse content
        return self.content_db.query(
            filter={"rating": {"$gte": 8.0}},
            sort="popularity",
            diversify_by="genre",
            limit=10
        )
```

---

## 8. Feedback Loop & Continuous Learning

### 8.1 Online Learning Architecture

```
User Interaction → Event Logging → Feature Engineering → Model Update → Deployment
                    ↓                                            ↑
              Offline Analysis ← Batch Processing ← Data Warehouse
```

### 8.2 Real-Time Feedback Integration

**System Architecture:**

```python
class OnlineLearningPipeline:
    def __init__(self):
        self.event_stream = KafkaConsumer("user-interactions")
        self.feature_store = RedisFeatureStore()
        self.model_server = ModelServer()

    def process_feedback(self, event: InteractionEvent):
        # 1. Extract features from event
        features = self.extract_features(event)

        # 2. Compute reward signal
        reward = self.compute_reward(event)

        # 3. Update online model (if applicable)
        if self.should_update_online(event):
            self.update_online_model(features, reward)

        # 4. Log for offline training
        self.log_training_example(features, reward)

        # 5. Update feature store
        self.feature_store.update_user_features(event.user_id, features)

    def compute_reward(self, event: InteractionEvent) -> float:
        """Compute reward signal from user interaction"""
        if event.action == "watch_complete":
            return 1.0
        elif event.action == "watch_partial":
            return event.completion_rate
        elif event.action == "like":
            return 0.8
        elif event.action == "skip":
            return -0.5
        elif event.action == "dislike":
            return -1.0
        else:
            return 0.0

    def should_update_online(self, event: InteractionEvent) -> bool:
        # Only update online for high-confidence signals
        return event.action in ["watch_complete", "like", "dislike"]

    def update_online_model(self, features: np.ndarray, reward: float):
        # Online gradient descent update
        self.model_server.partial_fit(features.reshape(1, -1), [reward])
```

### 8.3 Offline Batch Learning

**Scheduled Retraining:**

```python
class BatchRetrainingPipeline:
    def __init__(self):
        self.schedule = "daily"  # or "weekly"

    def retrain_models(self):
        # 1. Extract training data from data warehouse
        training_data = self.extract_training_data(
            start_date=datetime.now() - timedelta(days=30),
            end_date=datetime.now()
        )

        # 2. Validate data quality
        validation_report = self.validate_data(training_data)
        if not validation_report.is_valid:
            self.alert_team(validation_report.issues)
            return

        # 3. Train new model
        new_model = self.train_model(training_data)

        # 4. Evaluate on hold-out set
        metrics = self.evaluate_model(new_model, hold_out_data=self.get_holdout_data())

        # 5. Compare with production model
        production_model = self.model_server.get_production_model()
        production_metrics = self.evaluate_model(production_model, hold_out_data=self.get_holdout_data())

        # 6. Deploy if better
        if metrics["ndcg@10"] > production_metrics["ndcg@10"] * 1.02:  # 2% improvement threshold
            self.deploy_model(new_model)
            self.log_model_update(metrics, production_metrics)
        else:
            self.log_retraining_result("No improvement, keeping production model")
```

### 8.4 Active Learning

**Strategy: Query Uncertain Examples for Human Annotation**

```python
class ActiveLearner:
    def identify_uncertain_examples(self, model: Model, pool: List[Example], k: int = 100) -> List[Example]:
        """Identify examples where model is most uncertain"""
        predictions = model.predict_proba(pool)

        # Compute uncertainty (entropy)
        uncertainties = []
        for pred in predictions:
            entropy = -np.sum(pred * np.log(pred + 1e-10))
            uncertainties.append(entropy)

        # Select top-k most uncertain
        top_k_indices = np.argsort(uncertainties)[-k:]
        return [pool[i] for i in top_k_indices]

    def request_annotations(self, examples: List[Example]):
        # Send to human annotators
        for example in examples:
            self.annotation_queue.add(AnnotationTask(
                query=example.query,
                content=example.content,
                model_prediction=example.predicted_relevance,
                priority="high"
            ))
```

### 8.5 Reinforcement Learning from Human Feedback (RLHF)

**Approach: Learn from User Preferences**

```python
class RLHFTrainer:
    def __init__(self):
        self.reward_model = RewardModel()
        self.policy_model = PolicyModel()  # The recommendation model

    def train_reward_model(self, preference_data: List[PreferenceExample]):
        """
        preference_data = [
            {"query": "action movie", "chosen": content_A, "rejected": content_B},
            ...
        ]
        """
        for example in preference_data:
            # Reward model learns to predict which content user prefers
            chosen_score = self.reward_model(example.query, example.chosen)
            rejected_score = self.reward_model(example.query, example.rejected)

            # Loss: chosen should score higher than rejected
            loss = max(0, rejected_score - chosen_score + margin)
            self.reward_model.backward(loss)

    def train_policy_with_ppo(self):
        """Train recommendation policy using PPO"""
        for batch in self.sample_episodes():
            # Collect trajectories
            old_log_probs = self.policy_model.log_prob(batch.states, batch.actions)
            rewards = self.reward_model.predict(batch.states, batch.actions)

            # PPO update
            for _ in range(num_ppo_epochs):
                new_log_probs = self.policy_model.log_prob(batch.states, batch.actions)
                ratio = torch.exp(new_log_probs - old_log_probs)

                clipped_ratio = torch.clamp(ratio, 1 - epsilon, 1 + epsilon)
                loss = -torch.min(ratio * rewards, clipped_ratio * rewards).mean()

                self.policy_model.backward(loss)
```

### 8.6 Model Monitoring & Drift Detection

**Continuous Monitoring:**

```python
class ModelMonitor:
    def monitor_performance(self, window: timedelta = timedelta(hours=1)):
        # Compute real-time metrics
        current_metrics = self.compute_metrics(window)
        baseline_metrics = self.get_baseline_metrics()

        # Detect drift
        for metric_name, current_value in current_metrics.items():
            baseline_value = baseline_metrics[metric_name]

            if abs(current_value - baseline_value) / baseline_value > 0.1:  # 10% degradation
                self.alert_team(f"Performance drift detected in {metric_name}: {current_value} vs {baseline_value}")

                # Trigger investigation
                self.run_drift_analysis(metric_name)

    def run_drift_analysis(self, metric_name: str):
        """Analyze root cause of performance drift"""
        # Check data distribution shift
        current_data_dist = self.get_recent_data_distribution()
        baseline_data_dist = self.get_baseline_data_distribution()

        kl_divergence = self.compute_kl_divergence(current_data_dist, baseline_data_dist)

        if kl_divergence > threshold:
            self.alert_team(f"Data distribution shift detected: KL={kl_divergence}")

            # Suggest retraining
            self.suggest_retraining()
```

---

## 9. LLM Integration Options

### 9.1 Use Cases for LLMs

1. **Query Understanding & Expansion**
2. **Conversational Recommendations**
3. **Content Summarization & Explanations**
4. **Personalized Descriptions**
5. **Multi-Turn Dialogue**

### 9.2 Model Comparison

| Model | Provider | Strengths | Weaknesses | Cost | Latency |
|-------|----------|-----------|------------|------|---------|
| **GPT-4 Turbo** | OpenAI | Best reasoning, multi-modal | Expensive, rate limits | $10/$30 per 1M tokens | ~2-5s |
| **Claude 3.5 Sonnet** | Anthropic | Long context (200K), safety | Less widely adopted | $3/$15 per 1M tokens | ~1-3s |
| **Gemini 1.5 Pro** | Google | Multimodal, 1M context | Newer, less proven | $1.25/$5 per 1M tokens | ~2-4s |
| **Llama 3.1 70B** | Meta (Open) | Open-source, customizable | Self-hosting required | $0 (hosting cost) | ~500ms (optimized) |
| **Mistral Large** | Mistral | Good performance/cost | Smaller context (32K) | $2/$6 per 1M tokens | ~1-2s |

### 9.3 Recommended Architecture: Hybrid Approach

**Strategy: Use LLMs for complex reasoning, specialized models for speed**

```python
class HybridRecommendationEngine:
    def __init__(self):
        # Fast embedding-based search
        self.semantic_search = SemanticSearchEngine()

        # LLM for complex queries
        self.llm = AnthropicClaude("claude-3-5-sonnet-20241022")

        # Decision logic
        self.complexity_classifier = ComplexityClassifier()

    async def get_recommendations(self, query: str, user: UserProfile) -> List[Recommendation]:
        # Classify query complexity
        complexity = self.complexity_classifier.classify(query)

        if complexity == "simple":
            # Fast path: embedding-based search
            return self.semantic_search.search(query, user)

        elif complexity == "medium":
            # Hybrid: embeddings + LLM reranking
            candidates = self.semantic_search.search(query, user, k=50)
            reranked = await self.llm_rerank(query, candidates)
            return reranked[:10]

        else:  # complex
            # Full LLM path with reasoning
            return await self.llm_based_search(query, user)

    async def llm_based_search(self, query: str, user: UserProfile) -> List[Recommendation]:
        # Get candidate pool from semantic search
        candidates = self.semantic_search.search(query, user, k=100)

        # LLM analyzes query and ranks candidates
        prompt = f"""
        User query: "{query}"

        User profile:
        - Favorite genres: {user.favorite_genres}
        - Recent watches: {user.recent_watches}
        - Viewing context: {user.current_context}

        Available content (top 100):
        {self.format_candidates(candidates)}

        Task: Analyze the query, understand the user's intent and context, then rank the most relevant content.
        Provide:
        1. Query analysis (what is the user looking for?)
        2. Top 10 content recommendations with explanations
        3. Confidence score for each recommendation
        """

        response = await self.llm.complete(prompt)
        recommendations = self.parse_llm_response(response)

        return recommendations
```

### 9.4 LLM-Powered Query Expansion

```python
class LLMQueryExpander:
    def __init__(self):
        self.llm = OpenAI("gpt-4-turbo-preview")

    async def expand_query(self, query: str) -> ExpandedQuery:
        prompt = f"""
        Original user query: "{query}"

        Task: Expand this query to help retrieve more relevant entertainment content.

        Provide:
        1. Synonyms and related terms
        2. Implicit criteria (genres, moods, themes)
        3. Related movies/shows that match this query
        4. Temporal context (if applicable)

        Format as JSON.
        """

        response = await self.llm.complete(prompt, response_format={"type": "json_object"})
        expansion = json.loads(response.choices[0].message.content)

        return ExpandedQuery(
            original=query,
            synonyms=expansion["synonyms"],
            implicit_criteria=expansion["implicit_criteria"],
            related_content=expansion["related_content"],
            temporal_context=expansion.get("temporal_context")
        )
```

### 9.5 Conversational Recommendations

**Multi-Turn Dialogue System:**

```python
class ConversationalRecommender:
    def __init__(self):
        self.llm = AnthropicClaude("claude-3-5-sonnet-20241022")
        self.search_engine = SemanticSearchEngine()

    async def chat(self, user_message: str, conversation_history: List[Message], user: UserProfile) -> Response:
        # Build context from conversation history
        context = self.build_context(conversation_history, user)

        # LLM decides next action
        action_prompt = f"""
        Conversation history:
        {self.format_history(conversation_history)}

        User's latest message: "{user_message}"

        User profile summary:
        {self.summarize_profile(user)}

        Decide the next action:
        1. "search" - Enough information to search for content
        2. "clarify" - Need more information from user
        3. "refine" - Refine previous recommendations

        If search: Provide search query
        If clarify: Provide clarification question
        If refine: Provide refinement criteria

        Format as JSON: {{"action": "...", "payload": "..."}}
        """

        action_response = await self.llm.complete(action_prompt, response_format={"type": "json_object"})
        action = json.loads(action_response.choices[0].message.content)

        if action["action"] == "search":
            # Execute search
            recommendations = self.search_engine.search(action["payload"], user)

            # Generate natural language response
            response = await self.generate_response(user_message, recommendations)
            return Response(type="recommendations", recommendations=recommendations, message=response)

        elif action["action"] == "clarify":
            return Response(type="clarification", message=action["payload"])

        else:  # refine
            # Refine previous recommendations
            refined = self.refine_recommendations(action["payload"], context.previous_recommendations)
            response = await self.generate_response(user_message, refined)
            return Response(type="recommendations", recommendations=refined, message=response)
```

### 9.6 LLM-Generated Explanations

```python
class ExplanationGenerator:
    def __init__(self):
        self.llm = OpenAI("gpt-4-turbo-preview")

    async def explain_recommendation(self, query: str, content: Content, user: UserProfile, score: float) -> str:
        prompt = f"""
        User query: "{query}"

        Recommended content: {content.title} ({content.year})
        - Genres: {content.genres}
        - Description: {content.description}
        - Rating: {content.rating}/10

        User profile:
        - Favorite genres: {user.favorite_genres}
        - Recently watched: {user.recent_watches[-5:]}

        Relevance score: {score:.2f}

        Generate a concise, natural explanation (2-3 sentences) for why this content was recommended to the user.
        Focus on the connection between their query, preferences, and this specific content.
        """

        response = await self.llm.complete(prompt, max_tokens=100)
        explanation = response.choices[0].message.content.strip()

        return explanation

# Example output:
# "The Matrix perfectly matches your interest in mind-bending sci-fi with philosophical themes.
#  Like Inception which you enjoyed, it explores the nature of reality with stunning action sequences.
#  It's a classic from 1999 that influenced the genre you love."
```

### 9.7 Cost Optimization Strategies

**1. Caching:**
```python
class LLMCache:
    def __init__(self, redis_client):
        self.redis = redis_client
        self.ttl = 86400  # 24 hours

    async def get_or_compute(self, prompt: str, llm_function) -> str:
        cache_key = f"llm:{hashlib.sha256(prompt.encode()).hexdigest()}"
        cached = self.redis.get(cache_key)

        if cached:
            return cached.decode()

        result = await llm_function(prompt)
        self.redis.setex(cache_key, self.ttl, result)

        return result
```

**2. Prompt Optimization:**
- Use shorter prompts when possible
- Request JSON output (more token-efficient)
- Set lower max_tokens for bounded responses

**3. Model Selection:**
- Use cheaper models (Claude Haiku, GPT-3.5) for simple tasks
- Reserve expensive models (GPT-4, Claude Opus) for complex reasoning

**4. Batching:**
```python
async def batch_llm_requests(requests: List[str], batch_size: int = 10):
    # Combine multiple requests into one LLM call
    combined_prompt = "\n\n".join([f"Request {i+1}: {req}" for i, req in enumerate(requests)])
    response = await llm.complete(combined_prompt)

    # Parse individual responses
    return parse_batch_response(response)
```

### 9.8 Recommended LLM Strategy

**Primary:** Claude 3.5 Sonnet (Anthropic)
- **Reasoning:** Best balance of performance, cost, and speed
- **Use Cases:** Query understanding, conversational recommendations, explanations

**Secondary:** Llama 3.1 70B (Self-Hosted)
- **Reasoning:** Zero API cost, low latency, good quality
- **Use Cases:** High-volume tasks, embedding generation, classification

**Specialized:** GPT-4 Turbo (OpenAI)
- **Reasoning:** Best reasoning for complex edge cases
- **Use Cases:** Complex multi-criteria queries, ambiguity resolution

**Budget Allocation:**
- 70% of queries: Embedding-based search (no LLM)
- 20% of queries: Llama 3.1 (self-hosted)
- 8% of queries: Claude 3.5 Sonnet
- 2% of queries: GPT-4 Turbo (complex cases)

**Expected Monthly Cost (100K users, 1M queries):**
- Embedding models: ~$50 (infrastructure)
- Llama 3.1 hosting: ~$500 (GPU instances)
- Claude API: ~$600 (80K queries @ $3/$15 per 1M tokens)
- GPT-4 API: ~$200 (20K queries @ $10/$30 per 1M tokens)
- **Total:** ~$1,350/month

---

## 10. Performance Metrics & Monitoring

### 10.1 Key Performance Indicators (KPIs)

#### 10.1.1 Relevance Metrics

```python
@dataclass
class RelevanceMetrics:
    # Ranking metrics
    ndcg_at_10: float  # Normalized Discounted Cumulative Gain
    mrr: float  # Mean Reciprocal Rank
    precision_at_k: float  # Precision @ K
    recall_at_k: float  # Recall @ K

    # Business metrics
    click_through_rate: float  # CTR
    engagement_rate: float  # % who watch >5 min
    completion_rate: float  # % who finish content

    # Diversity metrics
    genre_diversity: float  # Entropy of genres in results
    temporal_diversity: float  # Year range coverage

    # User satisfaction
    avg_rating: float  # User feedback rating
    recommendation_acceptance: float  # % of recommendations acted upon
```

**Computation:**

```python
class MetricsCalculator:
    def compute_ndcg(self, relevance_scores: List[float], k: int = 10) -> float:
        """Normalized Discounted Cumulative Gain @ K"""
        # DCG = sum(rel_i / log2(i+1))
        dcg = sum([
            rel / np.log2(i + 2)  # i+2 because i is 0-indexed
            for i, rel in enumerate(relevance_scores[:k])
        ])

        # IDCG = DCG of perfect ranking
        ideal_scores = sorted(relevance_scores, reverse=True)[:k]
        idcg = sum([
            rel / np.log2(i + 2)
            for i, rel in enumerate(ideal_scores)
        ])

        return dcg / idcg if idcg > 0 else 0.0

    def compute_mrr(self, rankings: List[List[str]], relevant_items: List[str]) -> float:
        """Mean Reciprocal Rank"""
        reciprocal_ranks = []

        for ranking, relevant in zip(rankings, relevant_items):
            try:
                first_relevant_position = ranking.index(relevant) + 1
                reciprocal_ranks.append(1.0 / first_relevant_position)
            except ValueError:
                reciprocal_ranks.append(0.0)

        return np.mean(reciprocal_ranks)
```

#### 10.1.2 System Performance Metrics

```python
@dataclass
class SystemMetrics:
    # Latency
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float

    # Throughput
    queries_per_second: float

    # Resource utilization
    cpu_usage_percent: float
    memory_usage_gb: float
    gpu_usage_percent: float

    # Costs
    cost_per_1k_queries: float
    monthly_infrastructure_cost: float
```

#### 10.1.3 Model-Specific Metrics

```python
@dataclass
class ModelMetrics:
    # Embedding model
    embedding_quality: float  # Measured by retrieval@k
    embedding_inference_time_ms: float

    # Ranking model
    ranking_auc: float
    ranking_logloss: float

    # LLM (if used)
    llm_response_quality: float  # Human eval score
    llm_latency_ms: float
    llm_cost_per_query: float
```

### 10.2 Monitoring Dashboard

**Real-Time Monitoring Stack:**
- **Metrics:** Prometheus + Grafana
- **Logging:** ELK Stack (Elasticsearch, Logstash, Kibana)
- **Tracing:** Jaeger or Datadog APM
- **Alerting:** PagerDuty + Slack

**Dashboard Example:**

```python
# Prometheus metrics export
from prometheus_client import Counter, Histogram, Gauge

# Request metrics
search_requests_total = Counter('search_requests_total', 'Total search requests')
search_latency_seconds = Histogram('search_latency_seconds', 'Search latency in seconds')

# Quality metrics
ndcg_score = Gauge('ndcg_at_10', 'NDCG@10 score')
click_through_rate = Gauge('click_through_rate', 'Click-through rate')

# Error metrics
search_errors_total = Counter('search_errors_total', 'Total search errors', ['error_type'])

# Usage
@search_requests_total.count_exceptions()
@search_latency_seconds.time()
def search(query: str):
    results = search_engine.search(query)

    # Update quality metrics
    ndcg = compute_ndcg(results)
    ndcg_score.set(ndcg)

    return results
```

### 10.3 Alerting Rules

```yaml
# Prometheus alerting rules
groups:
  - name: recommendation_quality
    interval: 5m
    rules:
      - alert: LowNDCGScore
        expr: ndcg_at_10 < 0.7
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Recommendation quality degraded"
          description: "NDCG@10 has been below 0.7 for 15 minutes"

      - alert: HighSearchLatency
        expr: histogram_quantile(0.95, search_latency_seconds) > 1.0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Search latency too high"
          description: "p95 search latency > 1 second"

      - alert: LowCTR
        expr: click_through_rate < 0.15
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "Click-through rate dropped"
          description: "CTR has been below 15% for 30 minutes"
```

### 10.4 Experimentation Metrics

**A/B Test Scorecard:**

```python
class ExperimentScorecard:
    def generate_report(self, experiment_id: str) -> dict:
        control = self.get_variant_metrics(experiment_id, "control")
        treatment = self.get_variant_metrics(experiment_id, "treatment")

        return {
            "experiment_id": experiment_id,
            "duration_days": self.get_duration(experiment_id),
            "sample_size": {
                "control": control["sample_size"],
                "treatment": treatment["sample_size"]
            },
            "metrics": {
                "ndcg@10": {
                    "control": control["ndcg"],
                    "treatment": treatment["ndcg"],
                    "lift": (treatment["ndcg"] - control["ndcg"]) / control["ndcg"],
                    "p_value": self.compute_p_value(control["ndcg_values"], treatment["ndcg_values"]),
                    "significant": self.is_significant(control["ndcg_values"], treatment["ndcg_values"])
                },
                "ctr": {
                    "control": control["ctr"],
                    "treatment": treatment["ctr"],
                    "lift": (treatment["ctr"] - control["ctr"]) / control["ctr"],
                    "p_value": self.compute_p_value(control["ctr_values"], treatment["ctr_values"]),
                    "significant": self.is_significant(control["ctr_values"], treatment["ctr_values"])
                },
                "engagement_time": {
                    "control": control["engagement_time"],
                    "treatment": treatment["engagement_time"],
                    "lift": (treatment["engagement_time"] - control["engagement_time"]) / control["engagement_time"],
                    "p_value": self.compute_p_value(control["engagement_values"], treatment["engagement_values"]),
                    "significant": self.is_significant(control["engagement_values"], treatment["engagement_values"])
                }
            },
            "recommendation": self.make_decision(control, treatment)
        }
```

---

## 11. Implementation Roadmap

### Phase 1: MVP (Weeks 1-4)

**Objectives:**
- Basic semantic search functionality
- Simple content-based filtering
- Minimal viable product for testing

**Deliverables:**
1. NLP preprocessing pipeline (week 1)
2. Sentence-BERT embedding integration (week 1-2)
3. Vector database setup (Weaviate/Pinecone) (week 2)
4. Basic ranking with BM25 + embeddings (week 3)
5. Simple user profile (explicit preferences only) (week 3)
6. REST API for search (week 4)
7. Basic monitoring (week 4)

**Team:**
- 2 ML Engineers
- 1 Backend Engineer
- 1 DevOps Engineer

**Success Criteria:**
- NDCG@10 > 0.65
- Latency p95 < 1s
- 10K test queries processed

---

### Phase 2: Enhanced Recommendations (Weeks 5-8)

**Objectives:**
- Add collaborative filtering
- Implement hybrid recommendation
- Improve personalization

**Deliverables:**
1. Collaborative filtering model (ALS) (week 5)
2. Learning-to-rank model (LambdaMART) (week 6)
3. Hybrid ensemble (week 6)
4. Implicit user profiling (watch history, clicks) (week 7)
5. A/B testing framework (week 7)
6. First A/B test: hybrid vs semantic-only (week 8)

**Success Criteria:**
- NDCG@10 > 0.75
- CTR > 20%
- Engagement time > 3 min avg

---

### Phase 3: Advanced Features (Weeks 9-12)

**Objectives:**
- Context-aware recommendations
- LLM integration for complex queries
- Continuous learning pipeline

**Deliverables:**
1. Context enrichment (time, social, mood) (week 9)
2. LLM integration (Claude/GPT) for query expansion (week 10)
3. Conversational recommendations (multi-turn dialogue) (week 10-11)
4. Online learning pipeline (week 11)
5. Model monitoring & drift detection (week 12)
6. Active learning for edge cases (week 12)

**Success Criteria:**
- NDCG@10 > 0.82
- Complex query accuracy > 80%
- LLM response time < 3s

---

### Phase 4: Optimization & Scale (Weeks 13-16)

**Objectives:**
- Optimize for production scale
- Cost reduction
- Advanced personalization

**Deliverables:**
1. Model quantization & optimization (week 13)
2. Caching strategy (embeddings, LLM responses) (week 13)
3. Multi-model ensemble fine-tuning (week 14)
4. Diversity & exploration algorithms (MMR, bandits) (week 14)
5. Privacy-preserving personalization (differential privacy) (week 15)
6. Full production deployment (week 16)

**Success Criteria:**
- NDCG@10 > 0.85
- Latency p95 < 500ms
- Cost per 1K queries < $0.50
- Handle 1M users, 10M queries/day

---

## 12. Infrastructure & Deployment

### 12.1 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Client Layer                                │
│  (Web, Mobile, Voice Assistants)                                     │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       API Gateway (Kong/Nginx)                       │
│  - Rate limiting, auth, load balancing                               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Query Service  │ │  Recommendation │ │   User Service  │
│   (FastAPI)     │ │   Service       │ │   (Django)      │
│                 │ │   (Python)      │ │                 │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Vector Database │ │  Ranking Model  │ │  User Database  │
│ (Weaviate)      │ │   (MLflow)      │ │  (PostgreSQL)   │
└─────────────────┘ └─────────────────┘ └─────────────────┘
         │                   │
         │                   │
         ▼                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Event Streaming (Kafka)                           │
│  - User interactions, searches, watches                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Online Learning │ │  Feature Store  │ │   Analytics     │
│   Pipeline      │ │   (Feast)       │ │   (BigQuery)    │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

### 12.2 Technology Stack

**Core Services:**
- **API Layer:** FastAPI (Python) - High performance, async support
- **Vector Database:** Weaviate or Pinecone - Semantic search
- **User Database:** PostgreSQL - Relational data
- **Caching:** Redis - Fast lookups, session management
- **Message Queue:** Kafka - Event streaming
- **Feature Store:** Feast - ML feature management

**ML Infrastructure:**
- **Training:** Kubeflow or Vertex AI - Distributed training
- **Model Serving:** TorchServe or TensorFlow Serving
- **Experiment Tracking:** MLflow or Weights & Biases
- **Model Registry:** MLflow
- **Monitoring:** Prometheus + Grafana

**Cloud Platform (Google Cloud):**
- **Compute:** GKE (Google Kubernetes Engine)
- **ML:** Vertex AI for training and deployment
- **Storage:** Google Cloud Storage for model artifacts
- **Database:** Cloud SQL (PostgreSQL)
- **Analytics:** BigQuery

### 12.3 Deployment Strategy

**Containerization:**

```dockerfile
# Dockerfile for recommendation service
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY ./src /app/src
COPY ./models /app/models

# Expose port
EXPOSE 8000

# Run service
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

**Kubernetes Deployment:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: recommendation-service
spec:
  replicas: 5
  selector:
    matchLabels:
      app: recommendation-service
  template:
    metadata:
      labels:
        app: recommendation-service
    spec:
      containers:
      - name: recommendation-service
        image: gcr.io/project/recommendation-service:v1.0
        ports:
        - containerPort: 8000
        env:
        - name: VECTOR_DB_URL
          value: "weaviate-service:8080"
        - name: MODEL_PATH
          value: "/models/ranker_v1"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: recommendation-service
spec:
  selector:
    app: recommendation-service
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8000
  type: LoadBalancer
```

### 12.4 Scaling Strategy

**Horizontal Scaling:**
- Auto-scale pods based on CPU/memory/request rate
- Use Horizontal Pod Autoscaler (HPA) in Kubernetes

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: recommendation-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: recommendation-service
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**Database Scaling:**
- Read replicas for PostgreSQL
- Sharding for vector database
- Redis cluster for caching

### 12.5 Cost Estimation

**Monthly Infrastructure Costs (100K active users):**

| Component | Specification | Monthly Cost |
|-----------|--------------|--------------|
| **GKE Cluster** | 10 nodes (n2-standard-4) | $1,200 |
| **Vector Database** | Weaviate (self-hosted, 500GB) | $400 |
| **PostgreSQL** | Cloud SQL (db-n1-highmem-4) | $350 |
| **Redis** | Cloud Memorystore (5GB) | $150 |
| **Kafka** | Confluent Cloud (100GB) | $300 |
| **Model Serving** | Vertex AI Prediction (5 endpoints) | $600 |
| **BigQuery** | 1TB storage, 2TB queries | $250 |
| **LLM APIs** | Claude + GPT-4 (200K queries) | $1,000 |
| **Data Transfer** | 2TB egress | $200 |
| **Monitoring** | Datadog APM | $300 |
| **Total** | | **$4,750/month** |

**Cost per User:** ~$0.048/month
**Cost per Query:** ~$0.005

---

## Appendix A: Code Examples

### A.1 Complete Search Pipeline

```python
# Full recommendation pipeline implementation
class RecommendationPipeline:
    def __init__(self):
        # Initialize all components
        self.preprocessor = QueryPreprocessor()
        self.intent_classifier = IntentClassifier()
        self.entity_extractor = EntityExtractor()
        self.context_enricher = ContextEnricher()
        self.semantic_search = SemanticSearchEngine()
        self.collaborative_filter = CollaborativeFilteringRetriever()
        self.ranker = LearningToRankModel()
        self.diversifier = DiversificationEngine()

    async def search(self, query: str, user_id: str) -> List[Recommendation]:
        # 1. Preprocess query
        processed_query = self.preprocessor.process(query)

        # 2. Classify intent
        intent = self.intent_classifier.classify(processed_query)

        # 3. Extract entities
        entities = self.entity_extractor.extract(processed_query)

        # 4. Get user profile
        user = await self.get_user_profile(user_id)

        # 5. Enrich with context
        enriched_query = self.context_enricher.enrich(
            processed_query, user, self.get_current_session(user_id)
        )

        # 6. Candidate generation (parallel)
        semantic_candidates, collab_candidates = await asyncio.gather(
            self.semantic_search.retrieve(enriched_query, k=100),
            self.collaborative_filter.retrieve(user.id, k=100)
        )

        # 7. Merge and deduplicate candidates
        all_candidates = self.merge_candidates(
            semantic_candidates, collab_candidates
        )

        # 8. Rank candidates
        ranked = self.ranker.rank(enriched_query, all_candidates)

        # 9. Diversify results
        diversified = self.diversifier.apply_mmr(ranked, k=20)

        # 10. Generate explanations
        recommendations = [
            Recommendation(
                content=c.candidate.content,
                score=c.score,
                explanation=await self.generate_explanation(query, c, user)
            )
            for c in diversified[:10]
        ]

        # 11. Log for analytics
        await self.log_recommendations(user_id, query, recommendations)

        return recommendations
```

### A.2 Training Script

```python
# Training script for ranking model
import lightgbm as lgb
from sklearn.model_selection import train_test_split

def train_ranking_model(training_data_path: str, output_path: str):
    # Load training data
    data = load_training_data(training_data_path)

    # Split data
    X_train, X_val, y_train, y_val = train_test_split(
        data["features"],
        data["labels"],
        test_size=0.2,
        random_state=42
    )

    # Create query groups for ranking
    train_groups = data["train_groups"]
    val_groups = data["val_groups"]

    # Initialize LambdaMART model
    model = lgb.LGBMRanker(
        objective='lambdarank',
        metric='ndcg',
        ndcg_eval_at=[1, 3, 5, 10],
        boosting_type='gbdt',
        n_estimators=200,
        max_depth=8,
        learning_rate=0.05,
        num_leaves=63,
        min_child_samples=20,
        subsample=0.8,
        colsample_bytree=0.8,
        random_state=42
    )

    # Train model
    model.fit(
        X_train, y_train,
        group=train_groups,
        eval_set=[(X_val, y_val)],
        eval_group=[val_groups],
        eval_at=[1, 3, 5, 10],
        early_stopping_rounds=20,
        verbose=10
    )

    # Evaluate
    val_predictions = model.predict(X_val)
    ndcg = compute_ndcg(y_val, val_predictions, val_groups)
    print(f"Validation NDCG@10: {ndcg:.4f}")

    # Save model
    model.booster_.save_model(output_path)

    # Log to MLflow
    mlflow.log_metric("ndcg@10", ndcg)
    mlflow.sklearn.log_model(model, "ranking_model")

    return model
```

---

## Appendix B: References & Resources

### Academic Papers
1. "BERT: Pre-training of Deep Bidirectional Transformers" (Devlin et al., 2019)
2. "Sentence-BERT: Sentence Embeddings using Siamese BERT-Networks" (Reimers & Gurevych, 2019)
3. "Learning to Rank using Gradient Descent" (Burges et al., 2005)
4. "Collaborative Filtering for Implicit Feedback Datasets" (Hu et al., 2008)
5. "Deep Neural Networks for YouTube Recommendations" (Covington et al., 2016)

### Tools & Libraries
- **Transformers:** https://huggingface.co/transformers
- **Sentence-Transformers:** https://www.sbert.net
- **LightGBM:** https://lightgbm.readthedocs.io
- **Weaviate:** https://weaviate.io
- **Feast:** https://feast.dev

### APIs
- **TMDb API:** https://www.themoviedb.org/documentation/api
- **JustWatch API:** https://www.justwatch.com
- **Anthropic Claude:** https://www.anthropic.com/api
- **OpenAI GPT-4:** https://platform.openai.com

---

**Document End**

This comprehensive ML strategy provides a complete roadmap for building an intelligent media discovery system. The approach balances cutting-edge AI techniques with practical engineering considerations, ensuring scalability, cost-effectiveness, and exceptional user experience.

For questions or clarifications, contact the ML team at ml-team@example.com.
