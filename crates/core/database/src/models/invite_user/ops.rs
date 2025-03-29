use revolt_result::Result;

use crate::models::invite_user::model::InvitationToken;

mod mongodb;
// mod reference; // Uncomment when implementing reference DB support

#[async_trait]
pub trait AbstractInviteTokens: Sync + Send {
    /// Generate a new invitation token
    async fn generate_invite_token(&self, creator_id: String) -> Result<InvitationToken>;
}

// Re-export the trait from the ops module
pub use self::AbstractInviteTokens;
