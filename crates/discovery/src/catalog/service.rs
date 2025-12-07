use anyhow::{anyhow, Result};
use chrono::Utc;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use qdrant_client::Qdrant;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use super::types::{
    AvailabilityUpdate, ContentResponse, ContentType, CreateContentRequest, ImageSet,
    UpdateContentRequest,
};

pub struct CatalogService {
    db_pool: PgPool,
    qdrant_client: Arc<Qdrant>,
    kafka_producer: Option<Arc<FutureProducer>>,
    qdrant_collection: String,
    openai_api_key: String,
    openai_api_url: String,
}

impl CatalogService {
    pub fn new(
        db_pool: PgPool,
        qdrant_client: Arc<Qdrant>,
        qdrant_collection: String,
        openai_api_key: String,
        openai_api_url: String,
    ) -> Self {
        Self {
            db_pool,
            qdrant_client,
            kafka_producer: None,
            qdrant_collection,
            openai_api_key,
            openai_api_url,
        }
    }

    pub fn with_kafka(mut self, kafka_brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", kafka_brokers)
            .set("message.timeout.ms", "5000")
            .create()?;
        self.kafka_producer = Some(Arc::new(producer));
        Ok(self)
    }

    pub async fn create_content(&self, request: CreateContentRequest) -> Result<ContentResponse> {
        request.validate().map_err(|e| anyhow!(e))?;

        let content_id = Uuid::new_v4();
        let now = Utc::now();
        let content_type_str = content_type_to_string(&request.content_type);

        let result = sqlx::query(
            r#"
            INSERT INTO content (
                id, content_type, title, overview, release_date, runtime_minutes,
                popularity_score, average_rating, vote_count, created_at, last_updated
            )
            VALUES ($1, $2, $3, $4, $5, $6, 0.5, 0.0, 0, $7, $7)
            RETURNING id, created_at, last_updated
            "#,
        )
        .bind(content_id)
        .bind(content_type_str)
        .bind(&request.title)
        .bind(&request.overview)
        .bind(request.release_year.map(|y| format!("{}-01-01", y)))
        .bind(request.runtime_minutes)
        .bind(now)
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO platform_ids (content_id, platform, platform_content_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(content_id)
        .bind(&request.platform)
        .bind(&request.platform_content_id)
        .execute(&self.db_pool)
        .await?;

        for genre in &request.genres {
            sqlx::query(
                r#"
                INSERT INTO content_genres (content_id, genre)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(content_id)
            .bind(genre)
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(rating) = &request.rating {
            sqlx::query(
                r#"
                INSERT INTO content_ratings (content_id, region, rating)
                VALUES ($1, 'US', $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(content_id)
            .bind(rating)
            .execute(&self.db_pool)
            .await?;
        }

        let embedding = self
            .generate_embedding(&request.title, request.overview.as_deref())
            .await?;
        self.upsert_to_qdrant(content_id, &request.title, &request.genres, &embedding)
            .await?;

        self.emit_event("content.created", content_id, &request.title)
            .await?;

        Ok(ContentResponse {
            id: content_id,
            title: request.title,
            content_type: request.content_type,
            platform: request.platform,
            platform_content_id: request.platform_content_id,
            overview: request.overview,
            release_year: request.release_year,
            runtime_minutes: request.runtime_minutes,
            genres: request.genres,
            rating: request.rating,
            images: request.images,
            created_at: result.try_get("created_at").unwrap_or(now),
            updated_at: result.try_get("last_updated").unwrap_or(now),
            deleted_at: None,
        })
    }

    pub async fn get_content(&self, id: Uuid) -> Result<Option<ContentResponse>> {
        let record = sqlx::query(
            r#"
            SELECT
                c.id,
                c.content_type,
                c.title,
                c.overview,
                EXTRACT(YEAR FROM c.release_date)::int as release_year,
                c.runtime_minutes,
                c.created_at,
                c.last_updated,
                p.platform,
                p.platform_content_id,
                ARRAY_AGG(DISTINCT g.genre) FILTER (WHERE g.genre IS NOT NULL) as genres,
                r.rating
            FROM content c
            LEFT JOIN platform_ids p ON c.id = p.content_id
            LEFT JOIN content_genres g ON c.id = g.content_id
            LEFT JOIN content_ratings r ON c.id = r.content_id AND r.region = 'US'
            WHERE c.id = $1
            GROUP BY c.id, c.content_type, c.title, c.overview, c.release_date,
                     c.runtime_minutes, c.created_at, c.last_updated,
                     p.platform, p.platform_content_id, r.rating
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(record.map(|r| ContentResponse {
            id: r.try_get("id").unwrap(),
            title: r.try_get("title").unwrap(),
            content_type: parse_content_type(&r.try_get::<String, _>("content_type").unwrap()),
            platform: r.try_get("platform").unwrap_or_default(),
            platform_content_id: r.try_get("platform_content_id").unwrap_or_default(),
            overview: r.try_get("overview").ok(),
            release_year: r.try_get("release_year").ok(),
            runtime_minutes: r.try_get("runtime_minutes").ok(),
            genres: r.try_get::<Vec<String>, _>("genres").unwrap_or_default(),
            rating: r.try_get("rating").ok(),
            images: ImageSet::default(),
            created_at: r.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            updated_at: r.try_get("last_updated").unwrap_or_else(|_| Utc::now()),
            deleted_at: None,
        }))
    }

    pub async fn update_content(
        &self,
        id: Uuid,
        request: UpdateContentRequest,
    ) -> Result<ContentResponse> {
        let existing = self
            .get_content(id)
            .await?
            .ok_or_else(|| anyhow!("Content not found"))?;

        let title = request.title.clone().unwrap_or(existing.title.clone());
        let overview = request.overview.clone().or(existing.overview.clone());

        if request.title.is_some() || request.overview.is_some() {
            sqlx::query(
                r#"
                UPDATE content
                SET title = COALESCE($1, title),
                    overview = COALESCE($2, overview),
                    last_updated = $3
                WHERE id = $4
                "#,
            )
            .bind(&request.title)
            .bind(&request.overview)
            .bind(Utc::now())
            .bind(id)
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(genres) = &request.genres {
            sqlx::query(
                r#"
                DELETE FROM content_genres WHERE content_id = $1
                "#,
            )
            .bind(id)
            .execute(&self.db_pool)
            .await?;

            for genre in genres {
                sqlx::query(
                    r#"
                    INSERT INTO content_genres (content_id, genre)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(id)
                .bind(genre)
                .execute(&self.db_pool)
                .await?;
            }
        }

        if let Some(rating) = &request.rating {
            sqlx::query(
                r#"
                INSERT INTO content_ratings (content_id, region, rating)
                VALUES ($1, 'US', $2)
                ON CONFLICT (content_id, region) DO UPDATE SET rating = $2
                "#,
            )
            .bind(id)
            .bind(rating)
            .execute(&self.db_pool)
            .await?;
        }

        let embedding = self.generate_embedding(&title, overview.as_deref()).await?;
        let genres = request.genres.clone().unwrap_or(existing.genres.clone());
        self.upsert_to_qdrant(id, &title, &genres, &embedding)
            .await?;

        self.emit_event("content.updated", id, &title).await?;

        self.get_content(id)
            .await?
            .ok_or_else(|| anyhow!("Content not found after update"))
    }

    pub async fn delete_content(&self, id: Uuid) -> Result<()> {
        let content = self
            .get_content(id)
            .await?
            .ok_or_else(|| anyhow!("Content not found"))?;

        sqlx::query(
            r#"
            UPDATE content
            SET last_updated = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .execute(&self.db_pool)
        .await?;

        self.remove_from_qdrant(id).await?;

        self.emit_event("content.deleted", id, &content.title)
            .await?;

        Ok(())
    }

    pub async fn update_availability(&self, id: Uuid, update: AvailabilityUpdate) -> Result<()> {
        let content = self
            .get_content(id)
            .await?
            .ok_or_else(|| anyhow!("Content not found"))?;

        for region in &update.regions {
            let availability_type = if update.subscription_required {
                "subscription"
            } else if update.purchase_price.is_some() {
                "purchase"
            } else if update.rental_price.is_some() {
                "rental"
            } else {
                "free"
            };

            sqlx::query(
                r#"
                INSERT INTO platform_availability (
                    content_id, platform, region, availability_type,
                    price_cents, currency, deep_link, web_fallback,
                    available_from, expires_at
                )
                VALUES ($1, $2, $3, $4, $5, 'USD', '', '', $6, $7)
                "#,
            )
            .bind(id)
            .bind(&content.platform)
            .bind(region)
            .bind(availability_type)
            .bind(
                update
                    .purchase_price
                    .or(update.rental_price)
                    .map(|p| (p * 100.0) as i32),
            )
            .bind(update.available_from.unwrap_or_else(Utc::now))
            .bind(update.available_until)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    async fn generate_embedding(&self, title: &str, overview: Option<&str>) -> Result<Vec<f32>> {
        let text = if let Some(ov) = overview {
            format!("{} {}", title, ov)
        } else {
            title.to_string()
        };

        let client = reqwest::Client::new();
        let response = client
            .post(&self.openai_api_url)
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&json!({
                "input": text,
                "model": "text-embedding-3-small",
                "dimensions": 768
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid embedding response"))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect::<Vec<f32>>();

        if embedding.len() != 768 {
            return Err(anyhow!("Invalid embedding dimension: {}", embedding.len()));
        }

        Ok(embedding)
    }

    async fn upsert_to_qdrant(
        &self,
        id: Uuid,
        title: &str,
        genres: &[String],
        embedding: &[f32],
    ) -> Result<()> {
        let point = PointStruct::new(
            id.to_string(),
            embedding.to_vec(),
            json!({
                "title": title,
                "genres": genres,
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        self.qdrant_client
            .upsert_points(UpsertPointsBuilder::new(
                &self.qdrant_collection,
                vec![point],
            ))
            .await?;

        Ok(())
    }

    async fn remove_from_qdrant(&self, id: Uuid) -> Result<()> {
        use qdrant_client::qdrant::{DeletePointsBuilder, PointId};

        let points = vec![PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(
                id.to_string(),
            )),
        }];

        self.qdrant_client
            .delete_points(DeletePointsBuilder::new(&self.qdrant_collection).points(points))
            .await?;

        Ok(())
    }

    async fn emit_event(&self, event_type: &str, content_id: Uuid, title: &str) -> Result<()> {
        if let Some(producer) = &self.kafka_producer {
            let payload = json!({
                "event_type": event_type,
                "content_id": content_id.to_string(),
                "title": title,
                "timestamp": Utc::now().to_rfc3339(),
            })
            .to_string();

            let key = content_id.to_string();
            let record = FutureRecord::to("content-events")
                .key(&key)
                .payload(&payload);

            producer
                .send(record, std::time::Duration::from_secs(0))
                .await
                .map_err(|e| anyhow!("Failed to send Kafka event: {:?}", e))?;
        }

        Ok(())
    }
}

fn content_type_to_string(content_type: &ContentType) -> &'static str {
    match content_type {
        ContentType::Movie => "movie",
        ContentType::Series => "series",
        ContentType::Episode => "episode",
        ContentType::Short => "short",
        ContentType::Documentary => "documentary",
        ContentType::Special => "special",
    }
}

fn parse_content_type(s: &str) -> ContentType {
    match s {
        "movie" => ContentType::Movie,
        "series" => ContentType::Series,
        "episode" => ContentType::Episode,
        "short" => ContentType::Short,
        "documentary" => ContentType::Documentary,
        "special" => ContentType::Special,
        _ => ContentType::Movie,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(content_type_to_string(&ContentType::Movie), "movie");
        assert_eq!(content_type_to_string(&ContentType::Series), "series");
        assert_eq!(content_type_to_string(&ContentType::Episode), "episode");

        assert_eq!(parse_content_type("movie"), ContentType::Movie);
        assert_eq!(parse_content_type("series"), ContentType::Series);
        assert_eq!(parse_content_type("unknown"), ContentType::Movie);
    }
}
