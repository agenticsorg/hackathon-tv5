//! Context-Aware Filtering
//!
//! Filters recommendations based on temporal context, device type, and mood.

use crate::profile::UserProfile;
use crate::types::{RecommendationContext, RecommendationType, ScoredContent, TemporalContext};
use anyhow::{Context, Result};
use chrono::{Timelike, Utc};
use sqlx::{PgPool, Row};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Context-aware filtering engine
pub struct ContextAwareFilter {
    pool: PgPool,
}

impl ContextAwareFilter {
    /// Create new context-aware filter with database connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ContextAwareFilter {
    /// Generate candidates based on context
    pub async fn generate_candidates(
        &self,
        profile: &UserProfile,
        context: &RecommendationContext,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        let mut candidates = Vec::new();

        // Time-of-day filtering
        if let Some(time_of_day) = &context.time_of_day {
            let temporal_candidates = self
                .filter_by_time_of_day(profile, time_of_day, limit)
                .await?;
            candidates.extend(temporal_candidates);
        }

        // Device-type filtering
        if let Some(device_type) = &context.device_type {
            let device_candidates = self.filter_by_device(device_type, limit).await?;
            candidates.extend(device_candidates);
        }

        // Mood-based filtering
        if let Some(mood) = &context.mood {
            let mood_candidates = self.filter_by_mood(mood, limit).await?;
            candidates.extend(mood_candidates);
        }

        // Deduplicate and limit
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        candidates.dedup_by(|a, b| a.content_id == b.content_id);
        candidates.truncate(limit);

        Ok(candidates)
    }

    async fn filter_by_time_of_day(
        &self,
        profile: &UserProfile,
        time_of_day: &str,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        let start = std::time::Instant::now();

        // Get current hour
        let current_hour = Utc::now().hour() as usize;

        // Get user's historical preference for this hour
        let hour_preference = if current_hour < profile.temporal_patterns.hourly_patterns.len() {
            profile.temporal_patterns.hourly_patterns[current_hour]
        } else {
            0.5
        };

        // Map time_of_day to hour ranges
        let (hour_start, hour_end) = match time_of_day {
            "morning" => (6, 12),
            "afternoon" => (12, 18),
            "evening" => (18, 24),
            "night" => (0, 6),
            _ => (0, 24),
        };

        // Query content matching time-of-day viewing patterns
        // Join with watch_progress to find content typically watched during this time
        let query = r#"
            SELECT DISTINCT c.id, c.popularity_score
            FROM content c
            INNER JOIN watch_progress wp ON c.id = wp.content_id
            WHERE EXTRACT(HOUR FROM wp.last_watched) >= $1
              AND EXTRACT(HOUR FROM wp.last_watched) < $2
              AND wp.completion_rate > 0.3
            ORDER BY c.popularity_score DESC
            LIMIT $3
        "#;

        let rows = sqlx::query(query)
            .bind(hour_start as i32)
            .bind(hour_end as i32)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query content by time of day")?;

        let mut candidates = Vec::new();
        for row in rows {
            let content_id: Uuid = row.try_get("id")?;
            let popularity: f32 = row.try_get::<f64, _>("popularity_score")? as f32;

            // Calculate score: combine popularity with user's hourly preference
            let score = (popularity * 0.6 + hour_preference * 0.4).min(1.0);

            candidates.push(ScoredContent {
                content_id,
                score,
                source: RecommendationType::ContextAware,
                based_on: vec![format!("time_of_day:{}", time_of_day)],
            });
        }

        debug!(
            "Time-of-day filtering ({}) returned {} candidates in {:?}",
            time_of_day,
            candidates.len(),
            start.elapsed()
        );

        Ok(candidates)
    }

    async fn filter_by_device(
        &self,
        device_type: &crate::types::DeviceType,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        let start = std::time::Instant::now();

        // Map DeviceType enum to string for database query
        let device_str = match device_type {
            crate::types::DeviceType::TV => "tv",
            crate::types::DeviceType::Mobile => "mobile",
            crate::types::DeviceType::Desktop => "web",
            crate::types::DeviceType::Tablet => "mobile", // Treat tablet as mobile
        };

        // Query content appropriate for device type
        // - For TV: prefer longer runtime content
        // - For Mobile: prefer shorter runtime content
        // - For Desktop: any content
        let query = match device_type {
            crate::types::DeviceType::TV => {
                // TV prefers longer content (movies, full episodes)
                r#"
                    SELECT c.id, c.popularity_score, c.runtime_minutes
                    FROM content c
                    WHERE c.runtime_minutes >= 30
                      AND c.popularity_score > 0.3
                    ORDER BY c.popularity_score DESC, c.runtime_minutes DESC
                    LIMIT $1
                "#
            }
            crate::types::DeviceType::Mobile | crate::types::DeviceType::Tablet => {
                // Mobile/Tablet prefers shorter content
                r#"
                    SELECT c.id, c.popularity_score, c.runtime_minutes
                    FROM content c
                    WHERE c.runtime_minutes IS NOT NULL
                      AND c.runtime_minutes <= 60
                      AND c.popularity_score > 0.3
                    ORDER BY c.popularity_score DESC, c.runtime_minutes ASC
                    LIMIT $1
                "#
            }
            crate::types::DeviceType::Desktop => {
                // Desktop has no runtime preference
                r#"
                    SELECT c.id, c.popularity_score, c.runtime_minutes
                    FROM content c
                    WHERE c.popularity_score > 0.3
                    ORDER BY c.popularity_score DESC
                    LIMIT $1
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query content by device type")?;

        let mut candidates = Vec::new();
        for row in rows {
            let content_id: Uuid = row.try_get("id")?;
            let popularity: f32 = row.try_get::<f64, _>("popularity_score")? as f32;
            let runtime: Option<i32> = row.try_get("runtime_minutes").ok();

            // Calculate device compatibility score
            let device_score = match device_type {
                crate::types::DeviceType::TV => {
                    // Boost score for longer content on TV
                    if let Some(mins) = runtime {
                        if mins > 60 {
                            1.0
                        } else {
                            0.8
                        }
                    } else {
                        0.7
                    }
                }
                crate::types::DeviceType::Mobile | crate::types::DeviceType::Tablet => {
                    // Boost score for shorter content on mobile
                    if let Some(mins) = runtime {
                        if mins < 30 {
                            1.0
                        } else {
                            0.8
                        }
                    } else {
                        0.7
                    }
                }
                crate::types::DeviceType::Desktop => 0.9, // Neutral for desktop
            };

            let score = (popularity * 0.5 + device_score * 0.5).min(1.0);

            candidates.push(ScoredContent {
                content_id,
                score,
                source: RecommendationType::ContextAware,
                based_on: vec![format!("device_type:{}", device_str)],
            });
        }

        debug!(
            "Device filtering ({}) returned {} candidates in {:?}",
            device_str,
            candidates.len(),
            start.elapsed()
        );

        Ok(candidates)
    }

    async fn filter_by_mood(&self, mood: &str, limit: usize) -> Result<Vec<ScoredContent>> {
        let start = std::time::Instant::now();

        // Map mood to genres
        // Based on the content_moods and content_genres tables
        let genres = match mood.to_lowercase().as_str() {
            "happy" | "joyful" | "cheerful" => vec!["comedy", "family", "animation"],
            "sad" | "melancholy" | "somber" => vec!["drama", "romance"],
            "excited" | "energetic" | "pumped" => vec!["action", "adventure", "thriller"],
            "relaxed" | "calm" | "peaceful" => vec!["documentary", "nature", "lifestyle"],
            "scared" | "fearful" => vec!["horror", "thriller"],
            "romantic" | "loving" => vec!["romance", "drama"],
            "curious" | "intrigued" => vec!["documentary", "mystery", "sci-fi"],
            "nostalgic" | "reminiscent" => vec!["classic", "drama"],
            _ => vec!["drama", "comedy"], // Default fallback
        };

        // Check if content_moods table exists and has data
        // If not, fall back to genre-based filtering
        let use_mood_table = sqlx::query("SELECT COUNT(*) FROM content_moods LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .is_ok();

        let candidates = if use_mood_table {
            // Try mood-based query first
            let mood_query = r#"
                SELECT DISTINCT c.id, c.popularity_score
                FROM content c
                INNER JOIN content_moods cm ON c.id = cm.content_id
                WHERE LOWER(cm.mood) = LOWER($1)
                  AND c.popularity_score > 0.3
                ORDER BY c.popularity_score DESC
                LIMIT $2
            "#;

            let mood_rows = sqlx::query(mood_query)
                .bind(mood)
                .bind(limit as i32)
                .fetch_all(&self.pool)
                .await;

            if let Ok(rows) = mood_rows {
                if !rows.is_empty() {
                    rows.into_iter()
                        .filter_map(|row| {
                            let content_id: Uuid = row.try_get("id").ok()?;
                            let popularity: f32 =
                                row.try_get::<f64, _>("popularity_score").ok()? as f32;

                            Some(ScoredContent {
                                content_id,
                                score: (popularity * 0.9).min(1.0), // High confidence for mood match
                                source: RecommendationType::ContextAware,
                                based_on: vec![format!("mood:{}", mood)],
                            })
                        })
                        .collect()
                } else {
                    // Fall back to genre-based if no mood matches
                    self.filter_by_genres(&genres, limit).await?
                }
            } else {
                // Fall back to genre-based if mood query fails
                self.filter_by_genres(&genres, limit).await?
            }
        } else {
            // Use genre-based filtering
            self.filter_by_genres(&genres, limit).await?
        };

        debug!(
            "Mood filtering ({}) returned {} candidates in {:?}",
            mood,
            candidates.len(),
            start.elapsed()
        );

        Ok(candidates)
    }

    /// Helper method to filter by genres
    async fn filter_by_genres(&self, genres: &[&str], limit: usize) -> Result<Vec<ScoredContent>> {
        let query = r#"
            SELECT DISTINCT c.id, c.popularity_score, COUNT(*) as genre_match_count
            FROM content c
            INNER JOIN content_genres cg ON c.id = cg.content_id
            WHERE LOWER(cg.genre) = ANY($1)
              AND c.popularity_score > 0.3
            GROUP BY c.id, c.popularity_score
            ORDER BY genre_match_count DESC, c.popularity_score DESC
            LIMIT $2
        "#;

        let genre_array: Vec<String> = genres.iter().map(|g| g.to_lowercase()).collect();

        let rows = sqlx::query(query)
            .bind(&genre_array)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query content by genres")?;

        let mut candidates = Vec::new();
        for row in rows {
            let content_id: Uuid = row.try_get("id")?;
            let popularity: f32 = row.try_get::<f64, _>("popularity_score")? as f32;
            let genre_matches: i64 = row.try_get("genre_match_count")?;

            // Score based on popularity and number of matching genres
            let genre_score = (genre_matches as f32 / genres.len() as f32).min(1.0);
            let score = (popularity * 0.5 + genre_score * 0.5).min(1.0);

            candidates.push(ScoredContent {
                content_id,
                score,
                source: RecommendationType::ContextAware,
                based_on: vec![format!("genres:{}", genres.join(","))],
            });
        }

        Ok(candidates)
    }

    /// Calculate temporal score based on user's historical patterns
    pub fn calculate_temporal_score(
        temporal_patterns: &TemporalContext,
        current_hour: usize,
        current_weekday: usize,
    ) -> f32 {
        let hourly_score = if current_hour < temporal_patterns.hourly_patterns.len() {
            temporal_patterns.hourly_patterns[current_hour]
        } else {
            0.5
        };

        let weekday_score = if current_weekday < temporal_patterns.weekday_patterns.len() {
            temporal_patterns.weekday_patterns[current_weekday]
        } else {
            0.5
        };

        // Weighted average
        hourly_score * 0.6 + weekday_score * 0.4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_score_calculation() {
        let patterns = TemporalContext {
            hourly_patterns: vec![0.3; 24],
            weekday_patterns: vec![0.7; 7],
            seasonal_patterns: vec![0.5; 4],
            recent_bias: 0.8,
        };

        let score = ContextAwareFilter::calculate_temporal_score(&patterns, 14, 2);
        assert!((score - 0.46).abs() < 0.01); // 0.3*0.6 + 0.7*0.4
    }
}
