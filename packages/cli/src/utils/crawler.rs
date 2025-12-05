use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

#[allow(dead_code)]
pub struct Crawler {
    client: Client,
    visited: HashSet<String>,
    max_depth: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Page {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
}

#[allow(dead_code)]
impl Crawler {
    pub fn new(max_depth: usize) -> Self {
        Self {
            client: Client::builder()
                .user_agent("ARW-CLI/0.1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            visited: HashSet::new(),
            max_depth,
        }
    }

    pub async fn crawl(&mut self, start_url: &str) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        let mut to_visit = vec![(start_url.to_string(), 0)];

        while let Some((url, depth)) = to_visit.pop() {
            if depth > self.max_depth || self.visited.contains(&url) {
                continue;
            }

            self.visited.insert(url.clone());

            match self.fetch_page(&url).await {
                Ok(page) => {
                    // Add links to visit queue
                    for link in &page.links {
                        if !self.visited.contains(link) && self.is_same_domain(&url, link) {
                            to_visit.push((link.clone(), depth + 1));
                        }
                    }
                    pages.push(page);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch {}: {}", url, e);
                }
            }
        }

        Ok(pages)
    }

    async fn fetch_page(&self, url: &str) -> Result<Page> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch URL: {}", url))?;

        let html = response
            .text()
            .await
            .with_context(|| "Failed to read response body")?;

        let document = Html::parse_document(&html);

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());

        // Extract links
        let link_selector = Selector::parse("a[href]").unwrap();
        let links: Vec<String> = document
            .select(&link_selector)
            .filter_map(|el| {
                el.value().attr("href").and_then(|href| {
                    self.resolve_url(url, href).ok()
                })
            })
            .collect();

        Ok(Page {
            url: url.to_string(),
            title,
            content: html,
            links,
        })
    }

    fn resolve_url(&self, base: &str, href: &str) -> Result<String> {
        let base_url = Url::parse(base)?;
        let resolved = base_url.join(href)?;
        Ok(resolved.to_string())
    }

    fn is_same_domain(&self, base: &str, url: &str) -> bool {
        match (Url::parse(base), Url::parse(url)) {
            (Ok(base_url), Ok(target_url)) => {
                base_url.domain() == target_url.domain()
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawler_new() {
        let crawler = Crawler::new(5);
        assert_eq!(crawler.max_depth, 5);
        assert_eq!(crawler.visited.len(), 0);
    }

    #[test]
    fn test_crawler_new_zero_depth() {
        let crawler = Crawler::new(0);
        assert_eq!(crawler.max_depth, 0);
    }

    #[test]
    fn test_resolve_url_absolute() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path/page.html";
        let href = "https://other.com/page";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://other.com/page");
    }

    #[test]
    fn test_resolve_url_relative_path() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path/page.html";
        let href = "../other.html";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/other.html");
    }

    #[test]
    fn test_resolve_url_relative_simple() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path/";
        let href = "page.html";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/path/page.html");
    }

    #[test]
    fn test_resolve_url_root_relative() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path/page.html";
        let href = "/root/page.html";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/root/page.html");
    }

    #[test]
    fn test_resolve_url_invalid_base() {
        let crawler = Crawler::new(1);
        let base = "not-a-url";
        let href = "/page.html";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_same_domain_true() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/page1";
        let url = "https://example.com/page2";

        assert!(crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_false() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/page";
        let url = "https://other.com/page";

        assert!(!crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_subdomain() {
        let crawler = Crawler::new(1);
        let base = "https://www.example.com/page";
        let url = "https://api.example.com/page";

        assert!(!crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_invalid_base() {
        let crawler = Crawler::new(1);
        let base = "not-a-url";
        let url = "https://example.com/page";

        assert!(!crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_invalid_target() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/page";
        let url = "not-a-url";

        assert!(!crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_both_invalid() {
        let crawler = Crawler::new(1);
        let base = "not-a-url";
        let url = "also-not-a-url";

        assert!(!crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_page_creation() {
        let page = Page {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            content: "<html><body>Test</body></html>".to_string(),
            links: vec!["https://example.com/link".to_string()],
        };

        assert_eq!(page.url, "https://example.com");
        assert_eq!(page.title, Some("Example".to_string()));
        assert_eq!(page.content, "<html><body>Test</body></html>");
        assert_eq!(page.links.len(), 1);
    }

    #[test]
    fn test_page_clone() {
        let page = Page {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            content: "content".to_string(),
            links: vec![],
        };

        let cloned = page.clone();
        assert_eq!(page.url, cloned.url);
        assert_eq!(page.title, cloned.title);
    }

    #[test]
    fn test_crawler_visited_tracking() {
        let mut crawler = Crawler::new(5);
        assert_eq!(crawler.visited.len(), 0);

        crawler.visited.insert("https://example.com".to_string());
        assert_eq!(crawler.visited.len(), 1);
        assert!(crawler.visited.contains("https://example.com"));
    }

    #[test]
    fn test_resolve_url_with_query() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path";
        let href = "page.html?query=value";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("query=value"));
    }

    #[test]
    fn test_resolve_url_with_fragment() {
        let crawler = Crawler::new(1);
        let base = "https://example.com/path";
        let href = "page.html#section";

        let result = crawler.resolve_url(base, href);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("#section"));
    }

    #[test]
    fn test_is_same_domain_with_port() {
        let crawler = Crawler::new(1);
        let base = "https://example.com:8080/page";
        let url = "https://example.com:8080/other";

        assert!(crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_different_port() {
        let crawler = Crawler::new(1);
        let base = "https://example.com:8080/page";
        let url = "https://example.com:9090/page";

        // Different ports but same domain should still match
        assert!(crawler.is_same_domain(base, url));
    }

    #[test]
    fn test_is_same_domain_http_vs_https() {
        let crawler = Crawler::new(1);
        let base = "http://example.com/page";
        let url = "https://example.com/page";

        // Same domain, different protocols
        assert!(crawler.is_same_domain(base, url));
    }
}
