use revolt_quark::{
    models::{Channel, User},
    Database, Error, Ref, Result,
};
use rocket::{State, serde::json::Json};
use ulid::Ulid;

/// # Open Marketplace Direct Message
///
/// Buyer = authenticated user  
/// Seller = target user in URL  
/// Listing = opaque external identifier
/// Marketplace DMs cannot be opened with oneself.
/// If a DM already exists for the given listing between the buyer and seller, it is returned.
#[openapi(tag = "Marketplace")]
#[post("/marketplace/<listing_id>/dm/<seller>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    listing_id: String,
    seller: Ref,
) -> Result<Json<Channel>> {
    // Resolve seller user (same as open_dm)
    let seller = seller.as_user(db).await?;

    // Prevent self-DM
    if seller.id == user.id {
        return Err(Error::InvalidOperation);
    }

    if let Ok(existing) = db
        .find_marketplace_dm(&listing_id, &user.id, &seller.id)
        .await
    {
        return Ok(Json(existing));
    }

    // Create Marketplace DM
    let channel = Channel::MarketplaceDM {
        id: Ulid::new().to_string(),
        buyer: user.id.clone(),
        seller: seller.id.clone(),
        listing_id,
        last_message_id: None,
        recipients: vec![user.id.clone(), seller.id.clone()],
    };

    channel.create(db).await?;
    Ok(Json(channel))
}
