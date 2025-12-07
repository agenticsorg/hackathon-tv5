use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, ReloadPolicy};
use uuid::Uuid;

use super::filters::SearchFilters;
use super::{ContentSummary, SearchResult};

/// BM25 keyword search using Tantivy
pub struct KeywordSearch {
    index: Index,
    schema: Schema,
    index_path: String,
    top_k: usize,
    min_score: f32,
}

impl KeywordSearch {
    /// Create new keyword search instance
    pub fn new(index_path: String) -> Self {
        // Define schema
        let mut schema_builder = Schema::builder();

        let _id_field = schema_builder.add_text_field("id", STRING | STORED);
        let _title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let _overview_field = schema_builder.add_text_field("overview", TEXT | STORED);
        let _genres_field = schema_builder.add_text_field("genres", STRING | STORED);
        let _platforms_field = schema_builder.add_text_field("platforms", STRING | STORED);
        let _release_year_field = schema_builder.add_i64_field("release_year", INDEXED | STORED);
        let _popularity_field = schema_builder.add_f64_field("popularity_score", INDEXED | STORED);

        let schema = schema_builder.build();

        // Open or create index
        let index = match Index::open_in_dir(&index_path) {
            Ok(idx) => idx,
            Err(_) => {
                std::fs::create_dir_all(&index_path).expect("Failed to create index directory");
                Index::create_in_dir(&index_path, schema.clone()).expect("Failed to create index")
            }
        };

        Self {
            index,
            schema,
            index_path,
            top_k: 50,
            min_score: 0.5,
        }
    }

    /// Execute BM25 keyword search
    pub async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let searcher = reader.searcher();

        // Parse query
        let title_field = self.schema.get_field("title").unwrap();
        let overview_field = self.schema.get_field("overview").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![title_field, overview_field]);

        let parsed_query = query_parser.parse_query(query)?;

        // Execute search
        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(self.top_k))?;

        // Convert results
        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;

            let id_str = retrieved_doc
                .get_first(self.schema.get_field("id").unwrap())
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("");

            let id = Uuid::parse_str(id_str)?;

            let title = retrieved_doc
                .get_first(self.schema.get_field("title").unwrap())
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let overview = retrieved_doc
                .get_first(self.schema.get_field("overview").unwrap())
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let release_year = retrieved_doc
                .get_first(self.schema.get_field("release_year").unwrap())
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::I64(i) => Some(*i),
                    _ => None,
                })
                .unwrap_or(0) as i32;

            let genres: Vec<String> = retrieved_doc
                .get_all(self.schema.get_field("genres").unwrap())
                .filter_map(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();

            let platforms: Vec<String> = retrieved_doc
                .get_all(self.schema.get_field("platforms").unwrap())
                .filter_map(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();

            let popularity_score = retrieved_doc
                .get_first(self.schema.get_field("popularity_score").unwrap())
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::F64(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(0.0) as f32;

            let content = ContentSummary {
                id,
                title,
                overview,
                release_year,
                genres,
                platforms,
                popularity_score,
            };

            // Apply filters
            if let Some(ref filters) = filters {
                if !self.matches_filters(&content, filters) {
                    continue;
                }
            }

            results.push(SearchResult {
                content,
                relevance_score: _score,
                match_reasons: vec!["keyword_match".to_string()],
                vector_similarity: None,
                graph_score: None,
                keyword_score: Some(_score),
            });
        }

        Ok(results)
    }

    /// Check if content matches filters
    fn matches_filters(&self, content: &ContentSummary, filters: &SearchFilters) -> bool {
        // Genre filter
        if !filters.genres.is_empty() {
            let has_genre = content.genres.iter().any(|g| filters.genres.contains(g));
            if !has_genre {
                return false;
            }
        }

        // Platform filter
        if !filters.platforms.is_empty() {
            let has_platform = content
                .platforms
                .iter()
                .any(|p| filters.platforms.contains(p));
            if !has_platform {
                return false;
            }
        }

        // Year range filter
        if let Some((min_year, max_year)) = filters.year_range {
            if content.release_year < min_year || content.release_year > max_year {
                return false;
            }
        }

        true
    }

    /// Index a document
    pub fn index_document(&self, content: &ContentSummary) -> anyhow::Result<()> {
        let mut index_writer = self.index.writer(50_000_000)?;

        let mut doc = tantivy::TantivyDocument::default();

        doc.add_text(self.schema.get_field("id").unwrap(), content.id.to_string());
        doc.add_text(self.schema.get_field("title").unwrap(), &content.title);
        doc.add_text(
            self.schema.get_field("overview").unwrap(),
            &content.overview,
        );

        for genre in &content.genres {
            doc.add_text(self.schema.get_field("genres").unwrap(), genre);
        }

        for platform in &content.platforms {
            doc.add_text(self.schema.get_field("platforms").unwrap(), platform);
        }

        doc.add_i64(
            self.schema.get_field("release_year").unwrap(),
            content.release_year as i64,
        );
        doc.add_f64(
            self.schema.get_field("popularity_score").unwrap(),
            content.popularity_score as f64,
        );

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_str().unwrap().to_string();

        let keyword_search = KeywordSearch::new(index_path);

        // Index a document
        let content = ContentSummary {
            id: Uuid::new_v4(),
            title: "The Matrix".to_string(),
            overview: "A computer hacker learns about the true nature of reality".to_string(),
            release_year: 1999,
            genres: vec!["action".to_string(), "sci-fi".to_string()],
            platforms: vec!["netflix".to_string()],
            popularity_score: 0.9,
        };

        keyword_search.index_document(&content).unwrap();

        // Search
        let results = tokio_test::block_on(keyword_search.search("matrix", None)).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].content.title, "The Matrix");
    }
}
