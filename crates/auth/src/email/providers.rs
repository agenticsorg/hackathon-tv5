use super::service::{EmailError, EmailService, Result};
use super::templates::{EmailTemplate, TemplateEngine};
use async_trait::async_trait;
use reqwest;
use serde_json::json;

pub struct SendGridProvider {
    api_key: String,
    from_email: String,
    from_name: String,
    template_engine: TemplateEngine,
    client: reqwest::Client,
}

impl SendGridProvider {
    pub fn new(api_key: String, from_email: String, from_name: String, base_url: String) -> Self {
        Self {
            api_key,
            from_email: from_email.clone(),
            from_name: from_name.clone(),
            template_engine: TemplateEngine::new(base_url, from_name.clone()),
            client: reqwest::Client::new(),
        }
    }

    async fn send_email(&self, to_email: &str, template: EmailTemplate) -> Result<()> {
        let payload = json!({
            "personalizations": [{
                "to": [{"email": to_email}],
                "subject": template.subject
            }],
            "from": {
                "email": self.from_email,
                "name": self.from_name
            },
            "content": [
                {
                    "type": "text/plain",
                    "value": template.text_body
                },
                {
                    "type": "text/html",
                    "value": template.html_body
                }
            ]
        });

        let response = self
            .client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmailError::SendFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(EmailError::SendFailed(format!(
                "SendGrid API error: {}",
                error_text
            )))
        }
    }
}

#[async_trait]
impl EmailService for SendGridProvider {
    async fn send_verification(&self, email: &str, token: &str) -> Result<()> {
        let template = self.template_engine.render_verification(email, token)?;
        self.send_email(email, template).await
    }

    async fn send_password_reset(&self, email: &str, token: &str) -> Result<()> {
        let template = self.template_engine.render_password_reset(email, token)?;
        self.send_email(email, template).await
    }

    async fn send_password_changed(&self, email: &str) -> Result<()> {
        let template = self.template_engine.render_password_changed(email)?;
        self.send_email(email, template).await
    }
}

pub struct ConsoleProvider {
    template_engine: TemplateEngine,
}

impl ConsoleProvider {
    pub fn new(base_url: String, from_name: String) -> Self {
        Self {
            template_engine: TemplateEngine::new(base_url, from_name),
        }
    }

    fn print_email(&self, to_email: &str, template: EmailTemplate) {
        println!("\n{:=<60}", "");
        println!("EMAIL SENT (Console Provider - Development Mode)");
        println!("{:=<60}", "");
        println!("To: {}", to_email);
        println!("Subject: {}", template.subject);
        println!("{:-<60}", "");
        println!("{}", template.text_body);
        println!("{:=<60}\n", "");
    }
}

#[async_trait]
impl EmailService for ConsoleProvider {
    async fn send_verification(&self, email: &str, token: &str) -> Result<()> {
        let template = self.template_engine.render_verification(email, token)?;
        self.print_email(email, template);
        Ok(())
    }

    async fn send_password_reset(&self, email: &str, token: &str) -> Result<()> {
        let template = self.template_engine.render_password_reset(email, token)?;
        self.print_email(email, template);
        Ok(())
    }

    async fn send_password_changed(&self, email: &str) -> Result<()> {
        let template = self.template_engine.render_password_changed(email)?;
        self.print_email(email, template);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_console_provider_verification() {
        let provider = ConsoleProvider::new(
            "http://localhost:8080".to_string(),
            "Media Gateway".to_string(),
        );

        let result = provider
            .send_verification("test@example.com", "abc123")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_provider_password_reset() {
        let provider = ConsoleProvider::new(
            "http://localhost:8080".to_string(),
            "Media Gateway".to_string(),
        );

        let result = provider
            .send_password_reset("test@example.com", "xyz789")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_provider_password_changed() {
        let provider = ConsoleProvider::new(
            "http://localhost:8080".to_string(),
            "Media Gateway".to_string(),
        );

        let result = provider.send_password_changed("test@example.com").await;
        assert!(result.is_ok());
    }
}
