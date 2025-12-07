use super::service::Result;

pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

pub struct TemplateEngine {
    base_url: String,
    from_name: String,
}

impl TemplateEngine {
    pub fn new(base_url: String, from_name: String) -> Self {
        Self {
            base_url,
            from_name,
        }
    }

    pub fn render_verification(&self, email: &str, token: &str) -> Result<EmailTemplate> {
        let verification_url = format!("{}/verify?token={}", self.base_url, token);

        let subject = format!("Verify your {} account", self.from_name);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify Your Email</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background: #f9f9f9;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #e0e0e0;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #2c3e50;
            margin: 0;
        }}
        .content {{
            background: white;
            padding: 25px;
            border-radius: 6px;
            margin-bottom: 20px;
        }}
        .button {{
            display: inline-block;
            padding: 12px 30px;
            background: #3498db;
            color: white !important;
            text-decoration: none;
            border-radius: 4px;
            font-weight: 600;
            margin: 20px 0;
        }}
        .button:hover {{
            background: #2980b9;
        }}
        .footer {{
            text-align: center;
            color: #7f8c8d;
            font-size: 14px;
            margin-top: 20px;
        }}
        .security-note {{
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 15px;
            margin-top: 20px;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{}</h1>
        </div>

        <div class="content">
            <h2>Welcome!</h2>
            <p>Thank you for signing up. Please verify your email address to activate your account.</p>

            <p style="text-align: center;">
                <a href="{}" class="button">Verify Email Address</a>
            </p>

            <p style="color: #7f8c8d; font-size: 14px;">
                Or copy and paste this link into your browser:<br>
                <a href="{}">{}</a>
            </p>

            <div class="security-note">
                <strong>Security Note:</strong> This verification link will expire in 24 hours.
                If you didn't create an account with {}, please ignore this email.
            </div>
        </div>

        <div class="footer">
            <p>This is an automated message, please do not reply.</p>
            <p>&copy; 2024 {}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>"#,
            self.from_name,
            verification_url,
            verification_url,
            verification_url,
            self.from_name,
            self.from_name
        );

        let text_body = format!(
            r#"Welcome to {}!

Thank you for signing up. Please verify your email address to activate your account.

Click the link below to verify your email:
{}

This verification link will expire in 24 hours.

If you didn't create an account with {}, please ignore this email.

---
This is an automated message, please do not reply.
(c) 2024 {}. All rights reserved.
"#,
            self.from_name, verification_url, self.from_name, self.from_name
        );

        Ok(EmailTemplate {
            subject,
            html_body,
            text_body,
        })
    }

    pub fn render_password_reset(&self, email: &str, token: &str) -> Result<EmailTemplate> {
        let reset_url = format!("{}/reset-password?token={}", self.base_url, token);

        let subject = format!("Reset your {} password", self.from_name);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Your Password</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background: #f9f9f9;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #e0e0e0;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #2c3e50;
            margin: 0;
        }}
        .content {{
            background: white;
            padding: 25px;
            border-radius: 6px;
            margin-bottom: 20px;
        }}
        .button {{
            display: inline-block;
            padding: 12px 30px;
            background: #e74c3c;
            color: white !important;
            text-decoration: none;
            border-radius: 4px;
            font-weight: 600;
            margin: 20px 0;
        }}
        .button:hover {{
            background: #c0392b;
        }}
        .footer {{
            text-align: center;
            color: #7f8c8d;
            font-size: 14px;
            margin-top: 20px;
        }}
        .security-note {{
            background: #ffe0e0;
            border-left: 4px solid #e74c3c;
            padding: 15px;
            margin-top: 20px;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{}</h1>
        </div>

        <div class="content">
            <h2>Password Reset Request</h2>
            <p>We received a request to reset your password. Click the button below to create a new password.</p>

            <p style="text-align: center;">
                <a href="{}" class="button">Reset Password</a>
            </p>

            <p style="color: #7f8c8d; font-size: 14px;">
                Or copy and paste this link into your browser:<br>
                <a href="{}">{}</a>
            </p>

            <div class="security-note">
                <strong>Security Note:</strong> This password reset link will expire in 24 hours.
                If you didn't request a password reset, please ignore this email and your password will remain unchanged.
            </div>
        </div>

        <div class="footer">
            <p>This is an automated message, please do not reply.</p>
            <p>&copy; 2024 {}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>"#,
            self.from_name, reset_url, reset_url, reset_url, self.from_name
        );

        let text_body = format!(
            r#"Password Reset Request - {}

We received a request to reset your password.

Click the link below to create a new password:
{}

This password reset link will expire in 24 hours.

If you didn't request a password reset, please ignore this email and your password will remain unchanged.

---
This is an automated message, please do not reply.
(c) 2024 {}. All rights reserved.
"#,
            self.from_name, reset_url, self.from_name
        );

        Ok(EmailTemplate {
            subject,
            html_body,
            text_body,
        })
    }

    pub fn render_password_changed(&self, email: &str) -> Result<EmailTemplate> {
        let subject = format!("Your {} password was changed", self.from_name);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Changed</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background: #f9f9f9;
            border-radius: 8px;
            padding: 30px;
            border: 1px solid #e0e0e0;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #2c3e50;
            margin: 0;
        }}
        .content {{
            background: white;
            padding: 25px;
            border-radius: 6px;
            margin-bottom: 20px;
        }}
        .footer {{
            text-align: center;
            color: #7f8c8d;
            font-size: 14px;
            margin-top: 20px;
        }}
        .alert {{
            background: #e8f5e9;
            border-left: 4px solid #4caf50;
            padding: 15px;
            margin-top: 20px;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{}</h1>
        </div>

        <div class="content">
            <h2>Password Successfully Changed</h2>
            <p>Your password has been successfully changed.</p>

            <div class="alert">
                <strong>Security Notice:</strong> If you didn't make this change,
                please contact our support team immediately.
            </div>
        </div>

        <div class="footer">
            <p>This is an automated message, please do not reply.</p>
            <p>&copy; 2024 {}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>"#,
            self.from_name, self.from_name
        );

        let text_body = format!(
            r#"Password Successfully Changed - {}

Your password has been successfully changed.

Security Notice: If you didn't make this change, please contact our support team immediately.

---
This is an automated message, please do not reply.
(c) 2024 {}. All rights reserved.
"#,
            self.from_name, self.from_name
        );

        Ok(EmailTemplate {
            subject,
            html_body,
            text_body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_verification_email() {
        let engine = TemplateEngine::new(
            "https://example.com".to_string(),
            "Media Gateway".to_string(),
        );

        let template = engine
            .render_verification("test@example.com", "abc123def456")
            .unwrap();

        assert_eq!(template.subject, "Verify your Media Gateway account");
        assert!(template
            .html_body
            .contains("https://example.com/verify?token=abc123def456"));
        assert!(template
            .text_body
            .contains("https://example.com/verify?token=abc123def456"));
        assert!(template.html_body.contains("Welcome!"));
        assert!(template.text_body.contains("Welcome to Media Gateway!"));
    }

    #[test]
    fn test_render_password_reset_email() {
        let engine = TemplateEngine::new(
            "https://example.com".to_string(),
            "Media Gateway".to_string(),
        );

        let template = engine
            .render_password_reset("test@example.com", "xyz789uvw012")
            .unwrap();

        assert_eq!(template.subject, "Reset your Media Gateway password");
        assert!(template
            .html_body
            .contains("https://example.com/reset-password?token=xyz789uvw012"));
        assert!(template
            .text_body
            .contains("https://example.com/reset-password?token=xyz789uvw012"));
        assert!(template.html_body.contains("Password Reset Request"));
    }

    #[test]
    fn test_render_password_changed_email() {
        let engine = TemplateEngine::new(
            "https://example.com".to_string(),
            "Media Gateway".to_string(),
        );

        let template = engine.render_password_changed("test@example.com").unwrap();

        assert_eq!(template.subject, "Your Media Gateway password was changed");
        assert!(template.html_body.contains("Password Successfully Changed"));
        assert!(template.text_body.contains("Password Successfully Changed"));
        assert!(template.html_body.contains("Security Notice"));
    }
}
