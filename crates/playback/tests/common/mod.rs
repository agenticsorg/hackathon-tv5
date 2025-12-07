use media_gateway_playback::continue_watching::ContentMetadataProvider;
use uuid::Uuid;

/// Mock content metadata provider for testing
pub struct MockContentMetadataProvider;

#[async_trait::async_trait]
impl ContentMetadataProvider for MockContentMetadataProvider {
    async fn get_content_title(&self, content_id: Uuid, platform: &str) -> Result<String, String> {
        Ok(format!("Content {} on {}", content_id, platform))
    }
}
