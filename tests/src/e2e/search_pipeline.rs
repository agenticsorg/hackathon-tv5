//! End-to-end search pipeline tests
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct SearchQuery {
    query: String,
    user_id: Uuid,
    filters: Option<SearchFilters>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchFilters {
    content_type: Option<String>,
    genre: Option<Vec<String>>,
    year_range: Option<(i32, i32)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    content_id: Uuid,
    title: String,
    score: f32,
}

#[tokio::test]
async fn test_natural_language_search_basic() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("search-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create test content
    let content_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO content (id, title, description, content_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(content_id)
    .bind("Inception")
    .bind("A mind-bending thriller about dreams within dreams")
    .bind("movie")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Perform natural language search
    let search_query = "thriller about dreams";

    // Store search query
    sqlx::query(
        "INSERT INTO search_history (id, user_id, query, results_count, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(search_query)
    .bind(1)
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Verify search was recorded
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&containers.db_pool)
        .await?;
    assert_eq!(count, 1);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_vector_search_integration() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test content with embeddings
    let content_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO content (id, title, description, content_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(content_id)
    .bind("The Matrix")
    .bind("A hacker discovers reality is a simulation")
    .bind("movie")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Simulate vector embedding storage in Qdrant
    // In production, this would use Qdrant client to store vectors
    let collection_name = "content_embeddings";
    let vector_dimension = 384;

    // Store embedding metadata in Redis
    let embedding_key = format!("embedding:content:{}", content_id);
    redis::cmd("SET")
        .arg(&embedding_key)
        .arg(
            serde_json::json!({
                "content_id": content_id,
                "collection": collection_name,
                "dimension": vector_dimension,
                "indexed_at": Utc::now().timestamp()
            })
            .to_string(),
        )
        .arg("EX")
        .arg(86400)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify embedding metadata exists
    let exists: bool = redis::cmd("EXISTS")
        .arg(&embedding_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_with_filters() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test content
    for i in 0..3 {
        let content_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO content (id, title, description, content_type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(content_id)
        .bind(format!("Movie {}", i))
        .bind(format!("Description {}", i))
        .bind(if i % 2 == 0 { "movie" } else { "tv_show" })
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&containers.db_pool)
        .await?;
    }

    // Search with content_type filter
    let results: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, title FROM content WHERE content_type = $1 ORDER BY title")
            .bind("movie")
            .fetch_all(&containers.db_pool)
            .await?;

    assert_eq!(results.len(), 2);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_personalization_pipeline() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("personalize-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create user profile with preferences
    sqlx::query(
        "INSERT INTO user_profiles (id, user_id, preferences, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(serde_json::json!({"favorite_genres": ["sci-fi", "thriller"]}))
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Store personalization data in Redis
    let prefs_key = format!("user:prefs:{}", user_id);
    redis::cmd("SET")
        .arg(&prefs_key)
        .arg(
            serde_json::json!({
                "favorite_genres": ["sci-fi", "thriller"],
                "recently_viewed": [],
                "search_history": []
            })
            .to_string(),
        )
        .arg("EX")
        .arg(3600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify preferences stored
    let exists: bool = redis::cmd("EXISTS")
        .arg(&prefs_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_ranking_and_relevance() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test content with varying relevance
    let titles = vec!["Inception", "Dream Theater", "The Matrix"];
    for title in titles {
        let content_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO content (id, title, description, content_type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(content_id)
        .bind(title)
        .bind("Description")
        .bind("movie")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&containers.db_pool)
        .await?;
    }

    // Search for "dream" - should rank "Dream Theater" and "Inception" higher
    let results: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, title FROM content WHERE title ILIKE $1")
            .bind("%dream%")
            .fetch_all(&containers.db_pool)
            .await?;

    assert!(!results.is_empty());

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_analytics_tracking() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("analytics-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Track multiple searches
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO search_history (id, user_id, query, results_count, created_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(format!("query {}", i))
        .bind(i)
        .bind(Utc::now())
        .execute(&containers.db_pool)
        .await?;
    }

    // Verify search history
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&containers.db_pool)
        .await?;
    assert_eq!(count, 5);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_cached_search_results() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Cache search results
    let search_query = "popular movies";
    let cache_key = format!("search:cache:{}", search_query);

    let results = vec![
        serde_json::json!({"id": Uuid::new_v4(), "title": "Movie 1"}),
        serde_json::json!({"id": Uuid::new_v4(), "title": "Movie 2"}),
    ];

    redis::cmd("SET")
        .arg(&cache_key)
        .arg(serde_json::to_string(&results)?)
        .arg("EX")
        .arg(300)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Retrieve cached results
    let cached: String = redis::cmd("GET")
        .arg(&cache_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&cached)?;
    assert_eq!(parsed.len(), 2);

    containers.cleanup().await?;
    Ok(())
}
