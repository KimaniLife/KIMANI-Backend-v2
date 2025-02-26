use crate::Database;
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;
use serde_json::json;

/// Type of notification to send
#[derive(Debug, Clone)]
pub enum NotificationType {
    Email,
    SMS,
}

/// Task information for notifications
#[derive(Debug)]
struct NotificationTask {
    /// User IDs to notify
    recipients: Vec<String>,
    /// Notification content
    payload: String,
    /// Type of notification
    notification_type: NotificationType,
}

static Q: Lazy<Queue<NotificationTask>> = Lazy::new(|| Queue::new(10_000));

/// Queue a new notification task
pub async fn queue(recipients: Vec<String>, payload: String, notification_type: NotificationType) {
    if recipients.is_empty() {
        return;
    }

    Q.try_push(NotificationTask {
        recipients,
        payload,
        notification_type,
    })
    .ok();

    info!(
        "Notification queue is using {} slots from {}.",
        Q.len(),
        Q.capacity()
    );
}

/// Mock function to simulate sending email
async fn send_email(email: &str, content: &str) {
    info!("📧 Sending email to {}: {}", email, content);
}

/// Mock function to simulate sending SMS
async fn send_sms(phone: &str, content: &str) {
    info!("📱 Sending SMS to {}: {}", phone, content);
}

/// Start a notification worker
pub async fn worker(db: Database) {
    loop {
        let task = Q.pop().await;

        // Get user settings from database
        if let Ok(users) = db.fetch_users(&task.recipients).await {
            for user in users {
                // Fetch user settings which contain contact info
                if let Ok(settings) = db
                    .fetch_user_settings(
                        &user.id,
                        &["email".to_string(), "phone_number".to_string()],
                    )
                    .await
                {
                    match task.notification_type {
                        NotificationType::Email => {
                            if let Some((_, email)) = settings.get("email") {
                                send_email(email, &task.payload).await;
                            }
                        }
                        NotificationType::SMS => {
                            if let Some((_, phone)) = settings.get("phone_number") {
                                send_sms(phone, &task.payload).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
