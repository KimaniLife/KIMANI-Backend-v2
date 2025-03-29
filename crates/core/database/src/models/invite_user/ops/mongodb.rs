use chrono::{DateTime, Duration, Utc};
use mongodb::bson::doc;
use nanoid::nanoid;
use revolt_result::Result;

use crate::models::invite_user::model::InvitationToken;
use crate::models::invite_user::ops::AbstractInviteTokens;
use crate::MongoDb;

static COL: &str = "invite_tokens";

#[async_trait]
impl AbstractInviteTokens for MongoDb {
    /// Generate a new invitation token
    async fn generate_invite_token(&self, creator_id: String) -> Result<InvitationToken> {
        let token = InvitationToken {
            id: None,
            token_type: Some("invite".to_string()),
            expiry_date: Utc::now() + Duration::days(7),
            token: nanoid!(32),
            creator_id: creator_id,
        };

        query!(self, insert_one, COL, &token).map_err(|_| create_database_error!("insert", COL))?;

        Ok(token)
    }
}
