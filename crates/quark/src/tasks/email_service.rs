pub async fn send_email_via_sendgrid(
    sender_name: &str,
    sender_email: &str,
    recipient_name: &str,
    recipient_email: &str,
    content: &str,
    channel_id: &str,
    subject: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_key = std::env::var("SENDGRID_API_KEY")?;
    
    let body = serde_json::json!({
        "personalizations": [{
            "to": [{
                "email": recipient_email,
                "name": recipient_name
            }]
        }],
        "from": {
            "email": sender_email,
            "name": sender_name
        },
        "subject": subject,
        "content": [
            {
                "type": "text/html",
                "value": format!("{}<br><br/><a href=\"https://staging.kimanilife.com/{}\" style=\"text-decoration: underline; color: blue;\">Click here</a>", content, channel_id)
            }
        ]
    });
    
    let mut response = surf::post("https://api.sendgrid.com/v3/mail/send")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(surf::Body::from_json(&body)?)
        .await?;
    
    if response.status().is_success() {
        log::info!("Email sent successfully!");
    } else {
        let status = response.status();
        let text = response.body_string().await.unwrap_or_default();
        log::error!("Failed to send email. Status: {}, Body: {}", status, text);
    }
    
    Ok(())
}