use dotenv;
use revolt_quark::{
    models::{message::DataMessageSend, Message, User, Channel, Account},
    perms,
    types::push::MessageAuthor,
    web::idempotency::IdempotencyKey,
    Db, Error, Permission, Ref, Result, Database
};

use rocket::serde::json::Json;
use validator::Validate;

use revolt_quark::tasks::email_service::send_email_via_sendgrid;
use std::env;

/// # Send Message
///
/// Sends a message to the given channel.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages", data = "<data>")]
pub async fn message_send(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataMessageSend>,
    idempotency: IdempotencyKey,
) -> Result<Json<Message>> {

    dotenv::dotenv().ok();
    let data = data.into_inner();
    // let message_content = data.content.clone();
    // let content = message_content.as_deref().unwrap_or("");
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // Ensure we have permissions to send a message
    let channel = target.as_channel(db).await?;

    // Get recipients based on channel type
    let dm_recipient_id = match &channel {
        Channel::DirectMessage { recipients, .. } => {
            // Find the recipient that is not the current user
            recipients.iter()
                .find(|&id| id != &user.id)
                .cloned()
        },
        _ => None,
    };
    // let dm_recipient = target.as_user(db).await?;
    // let dm_recipient_account = target.as_user_account(db).await?;
    
    // Now dm_recipient_id will be Some(String) with the other user's ID in a DM,
    // or None if it's not a DM or if somehow the user is the only one in recipients
  
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .throw_permission_and_view_channel(db, Permission::SendMessage)
        .await?;

    // Verify permissions for masquerade
    if let Some(masq) = &data.masquerade {
        permissions
            .throw_permission(db, Permission::Masquerade)
            .await?;

        if masq.colour.is_some() {
            permissions
                .throw_permission(db, Permission::ManageRole)
                .await?;
        }
    }

    // Check permissions for embeds
    if data.embeds.as_ref().is_some_and(|v| !v.is_empty()) {
        permissions
            .throw_permission(db, Permission::SendEmbeds)
            .await?;
    }

    // Check permissions for files
    if data.attachments.as_ref().is_some_and(|v| !v.is_empty()) {
        permissions
            .throw_permission(db, Permission::UploadFiles)
            .await?;
    }

    // Ensure interactions information is correct
    if let Some(interactions) = &data.interactions {
        interactions.validate(db, &mut permissions).await?;
    }
    
    let message = channel
    .send_message(
        db,
        data,
        MessageAuthor::User(&user),
        idempotency,
        permissions
            .has_permission(db, Permission::SendEmbeds)
            .await?,
    )
    .await?;
    
    if let Channel::DirectMessage { id, .. } = &channel {
        let channel_url = format!("channel/{}", id);

        if let Some(dm_recipient_id) = dm_recipient_id {
            let dm_recipient = target.as_custom_user(&dm_recipient_id, db).await?;
            info!("DM is sending from user {} to recipient {:?}", &user.id, &dm_recipient);
            
            // Access the MongoDB client
            if let Database::MongoDb(mongo_db) = &**db {
                match mongo_db.find_one_by_id::<Account>("accounts", &dm_recipient_id).await {
                    Ok(account) => {
                        info!("Recipient's email is {:?}", account.email);
                        let sender_name = env::var("SENDER_NAME").unwrap();
                        let sender_email = env::var("SENDER_EMAIL").unwrap();
                        let recipient_name = &dm_recipient.username;
                        let recipient_email = env::var("RECIPIENT_EMAIL").unwrap();
                        let content = format!("{} sent 1 message", user.username);
                        let subject = "You have unread messages";
                        
                        match send_email_via_sendgrid(
                            &sender_name, 
                            &sender_email, 
                            recipient_name, 
                            &recipient_email, 
                            &content, 
                            &channel_url,
                            subject
                        ).await {
                            Ok(_) => {
                                info!("Email sent successfully to {}", recipient_email);
                            },
                            Err(err) => {
                                info!("Failed to send email: {:?}", err);
                            }
                        }
                    },
                    Err(err) => {
                        info!("Failed to fetch recipient's account: {:?}", err);
                    }
                }
            }
        }
    }

    // Return the message
    Ok(Json(message))
}
