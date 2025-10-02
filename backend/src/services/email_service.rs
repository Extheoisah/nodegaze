use crate::config::EmailConfig;
use crate::errors::{ServiceError, ServiceResult};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::str::FromStr;

pub struct EmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    config: EmailConfig,
}

impl EmailService {
    /// Creates a new EmailService instance
    pub fn new(config: EmailConfig) -> ServiceResult<Self> {
        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|e| ServiceError::validation(format!("Invalid SMTP host: {e}")))?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        Ok(Self { mailer, config })
    }

    /// Sends an invite email to the specified recipient
    pub async fn send_invite_email(
        &self,
        recipient_email: &str,
        recipient_name: Option<&str>,
        invite_token: &str,
        inviter_name: &str,
        account_name: &str,
    ) -> ServiceResult<()> {
        let subject = format!("You've been invited to join {account_name}");
        let invite_url = format!(
            "{}/accept-invite?token={}",
            self.config.base_url, invite_token
        );

        let html_content = self.build_invite_html(
            recipient_name.unwrap_or("there"),
            inviter_name,
            account_name,
            &invite_url,
        );

        let text_content = self.build_invite_text(
            recipient_name.unwrap_or("there"),
            inviter_name,
            account_name,
            &invite_url,
        );

        self.send_email(recipient_email, &subject, &html_content, &text_content)
            .await
    }

    /// Sends a generic email
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> ServiceResult<()> {
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.config.from_name, self.config.from_email
        ))
        .map_err(|e| ServiceError::validation(format!("Invalid from email: {e}")))?;

        let to_mailbox = Mailbox::from_str(to_email)
            .map_err(|e| ServiceError::validation(format!("Invalid recipient email: {e}")))?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_content.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_content.to_string()),
                    ),
            )
            .map_err(|e| ServiceError::validation(format!("Failed to build email: {e}")))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| ServiceError::validation(format!("Failed to send email: {e}")))?;

        Ok(())
    }

    fn build_invite_html(
        &self,
        recipient_name: &str,
        inviter_name: &str,
        account_name: &str,
        invite_url: &str,
    ) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>Invitation to join {}</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <h2 style="color: #2c3e50;">You've been invited!</h2>
                    
                    <p>Hi {},</p>
                    
                    <p><strong>{}</strong> has invited you to join <strong>{}</strong>.</p>
                    
                    <p>Click the button below to accept your invitation:</p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{}" 
                           style="background-color: #3498db; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block;">
                            Accept Invitation
                        </a>
                    </div>
                    
                    <p>Or copy and paste this link into your browser:</p>
                    <p style="word-break: break-all; color: #7f8c8d;">{}</p>
                    
                    <hr style="border: none; border-top: 1px solid #ecf0f1; margin: 30px 0;">
                    
                    <p style="font-size: 12px; color: #7f8c8d;">
                        This invitation will expire in 72 hours. If you didn't expect this invitation, 
                        you can safely ignore this email.
                    </p>
                </div>
            </body>
            </html>
            "#,
            account_name, recipient_name, inviter_name, account_name, invite_url, invite_url
        )
    }

    fn build_invite_text(
        &self,
        recipient_name: &str,
        inviter_name: &str,
        account_name: &str,
        invite_url: &str,
    ) -> String {
        format!(
            r#"You've been invited!

Hi {},

{} has invited you to join {}.

Click the link below to accept your invitation:
{}

This invitation will expire in 72 hours. If you didn't expect this invitation, you can safely ignore this email.
            "#,
            recipient_name, inviter_name, account_name, invite_url
        )
    }
}
