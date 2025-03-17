use revolt_quark::models::{User,Account};
use revolt_quark::{Database, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;

use revolt_quark::tasks::email_service::send_email_via_sendgrid;
use std::env;

/// # Accept Friend Request
///
/// Accept another user's friend request.
#[openapi(tag = "Relationships")]
#[put("/<target>/friend")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    let mut target = target.as_user(db).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(Error::IsBot);
    }

    if let Database::MongoDb(mongo_db) = &**db {

        match mongo_db.find_one_by_id::<Account>("accounts", &target.id).await {
         Ok(account) => {
            info!("FR Accepter's email is {:?}", account.email);
            let sender_name = env::var("SENDER_NAME").unwrap();
            let sender_email = env::var("SENDER_EMAIL").unwrap();
            let recipient_name = &target.username;
            // let recipient_email = account.email;
            let recipient_email = env::var("RECIPIENT_EMAIL").unwrap();
            let email_content =  format!("{} accepted your friend request", user.username);
            let content_string = match serde_json::to_string(&email_content) {
                Ok(s) => s,
                Err(e) => {
                    info!("Failed to serialize JSON: {:?}", e);
                    return Ok(Json(target));
                }
            };
            let channel_id = "friends";
            let subject = "Friend Request";
            match send_email_via_sendgrid(&sender_name, &sender_email, recipient_name, &recipient_email, &content_string, &channel_id, subject).await {
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

    info!("{} accepted your Friend Request", &target.username);
    user.add_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
