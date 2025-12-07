//! Full-text search using Tantivy

use crate::error::SearchResult;
use crate::{MatchType, SearchHit, SearchOptions};
use obsidian_core::note::Note;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};
use tracing::{debug, info};

/// Schema fields for the index
struct IndexFields {
    path: Field,
    title: Field,
    content: Field,
    headings: Field,
    tags: Field,
}

/// Full-text search index
pub struct FullTextIndex {
    index: Index,
    writer: Arc<RwLock<IndexWriter>>,
    reader: IndexReader,
    fields: IndexFields,
    schema: Schema,
}

impl FullTextIndex {
    /// Open or create an index at the given path
    pub fn open(path: impl Into<PathBuf>) -> SearchResult<Self> {
        let path = path.into();

        // Build schema
        let mut schema_builder = Schema::builder();

        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let headings_field = schema_builder.add_text_field("headings", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);

        let schema = schema_builder.build();

        // Create or open index
        std::fs::create_dir_all(&path)?;
        let index = Index::create_in_dir(&path, schema.clone())
            .or_else(|_| Index::open_in_dir(&path))?;

        // Create writer
        let writer = index.writer(50_000_000)?; // 50MB buffer

        // Create reader
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let fields = IndexFields {
            path: path_field,
            title: title_field,
            content: content_field,
            headings: headings_field,
            tags: tags_field,
        };

        info!("Opened full-text index at {:?}", path);

        Ok(Self {
            index,
            writer: Arc::new(RwLock::new(writer)),
            reader,
            fields,
            schema,
        })
    }

    /// Create an in-memory index for testing
    pub fn in_memory() -> SearchResult<Self> {
        let mut schema_builder = Schema::builder();

        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let headings_field = schema_builder.add_text_field("headings", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        let writer = index.writer(50_000_000)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let fields = IndexFields {
            path: path_field,
            title: title_field,
            content: content_field,
            headings: headings_field,
            tags: tags_field,
        };

        Ok(Self {
            index,
            writer: Arc::new(RwLock::new(writer)),
            reader,
            fields,
            schema,
        })
    }

    /// Index a note
    pub fn index_note(&self, note: &Note) -> SearchResult<()> {
        debug!("Indexing note: {}", note.id);

        // Remove existing document first
        self.remove_note(&note.id)?;

        // Get metadata
        let title = note
            .frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| note.basename.clone());

        let headings = note
            .frontmatter
            .as_ref()
            .and_then(|f| f.extra.get("headings"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let tags = note
            .frontmatter
            .as_ref()
            .map(|f| f.tags.join(" "))
            .unwrap_or_default();

        // Add document
        let writer = self.writer.write();
        writer.add_document(doc!(
            self.fields.path => note.id.clone(),
            self.fields.title => title,
            self.fields.content => note.content.clone(),
            self.fields.headings => headings,
            self.fields.tags => tags,
        ))?;

        Ok(())
    }

    /// Remove a note from the index
    pub fn remove_note(&self, path: &str) -> SearchResult<()> {
        debug!("Removing note from index: {}", path);

        let term = tantivy::Term::from_field_text(self.fields.path, path);
        let writer = self.writer.write();
        writer.delete_term(term);

        Ok(())
    }

    /// Commit changes to the index
    pub fn commit(&self) -> SearchResult<()> {
        debug!("Committing index changes");
        let mut writer = self.writer.write();
        writer.commit()?;
        Ok(())
    }

    /// Get a searcher for the index
    pub fn searcher(&self) -> SearchResult<FullTextSearcher> {
        let searcher = self.reader.searcher();
        Ok(FullTextSearcher {
            searcher,
            index: self.index.clone(),
            fields: IndexFields {
                path: self.fields.path,
                title: self.fields.title,
                content: self.fields.content,
                headings: self.fields.headings,
                tags: self.fields.tags,
            },
            schema: self.schema.clone(),
        })
    }

    /// Get the number of documents in the index
    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }
}

/// Full-text searcher
pub struct FullTextSearcher {
    searcher: tantivy::Searcher,
    index: Index,
    fields: IndexFields,
    schema: Schema,
}

impl FullTextSearcher {
    /// Search the index
    pub fn search(&self, query: &str, options: &SearchOptions) -> SearchResult<Vec<SearchHit>> {
        debug!("Searching for: {}", query);

        // Build list of fields to search
        let mut search_fields = Vec::new();
        if options.search_titles {
            search_fields.push(self.fields.title);
        }
        if options.search_content {
            search_fields.push(self.fields.content);
            search_fields.push(self.fields.headings);
        }
        if options.search_tags {
            search_fields.push(self.fields.tags);
        }

        if search_fields.is_empty() {
            search_fields.push(self.fields.content);
        }

        // Create query parser
        let query_parser = QueryParser::for_index(&self.index, search_fields);
        let parsed_query = query_parser.parse_query(query)?;

        // Search
        let limit = options.limit.unwrap_or(100);
        let top_docs = self.searcher.search(&parsed_query, &TopDocs::with_limit(limit))?;

        // Convert to search hits
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = self.searcher.doc(doc_address)?;

            let path = Self::extract_text(&retrieved_doc, self.fields.path);
            let title = Self::extract_text(&retrieved_doc, self.fields.title);
            let content = Self::extract_text(&retrieved_doc, self.fields.content);

            // Generate snippet
            let snippet = Self::generate_snippet(&content, query);

            let hit = SearchHit::new(path, score, title)
                .with_snippet(snippet)
                .with_match_type(MatchType::Content);

            results.push(hit);
        }

        Ok(results)
    }

    /// Extract text from a document field
    fn extract_text(doc: &tantivy::TantivyDocument, field: Field) -> String {
        doc.get_first(field)
            .map(|v| format!("{:?}", v))
            .map(|s| {
                // Remove the quotes from the debug format if present
                if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    s[1..s.len()-1].to_string()
                } else {
                    s
                }
            })
            .unwrap_or_default()
    }

    /// Generate a snippet from content
    fn generate_snippet(content: &str, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        // Find first occurrence
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());

            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&content[start..end]);
            if end < content.len() {
                snippet.push_str("...");
            }

            // Clean up newlines
            snippet.replace('\n', " ")
        } else {
            // Return first 100 chars
            let end = content.len().min(100);
            let mut snippet = content[..end].to_string();
            if content.len() > 100 {
                snippet.push_str("...");
            }
            snippet.replace('\n', " ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_note(path: &str, title: &str, content: &str) -> Note {
        use obsidian_core::note::Frontmatter;
        let mut note = Note::new(path, path, content);
        let mut fm = Frontmatter::new();
        fm.title = Some(title.to_string());
        note.frontmatter = Some(fm);
        note
    }

    #[test]
    fn test_index_and_search() {
        let index = FullTextIndex::in_memory().unwrap();

        let note1 = create_test_note("note1.md", "Hello World", "This is a test document about Rust programming.");
        let note2 = create_test_note("note2.md", "Goodbye World", "This is another document about Python.");

        index.index_note(&note1).unwrap();
        index.index_note(&note2).unwrap();
        index.commit().unwrap();

        let searcher = index.searcher().unwrap();
        let options = SearchOptions::quick();

        let results = searcher.search("Rust", &options).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].path.to_string_lossy().contains("note1"));

        let results = searcher.search("document", &options).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_remove_note() {
        let index = FullTextIndex::in_memory().unwrap();

        let note = create_test_note("test.md", "Test", "Test content");
        index.index_note(&note).unwrap();
        index.commit().unwrap();

        assert_eq!(index.num_docs(), 1);

        index.remove_note("test.md").unwrap();
        index.commit().unwrap();

        assert_eq!(index.num_docs(), 0);
    }

    #[test]
    fn test_snippet_generation() {
        let content = "This is a long document that contains the word Rust somewhere in the middle of the text for testing purposes.";
        let snippet = FullTextSearcher::generate_snippet(content, "Rust");

        assert!(snippet.contains("Rust"));
    }
}
