use revolt_database::{models::invite_user::model::InvitationToken, Database, DatabaseTrait};
use revolt_models::v0::ApiResponse;
use revolt_quark::models::User;
use revolt_result::Result;
use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use revolt_rocket_okapi::{openapi, JsonSchema};
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::{Deserialize, Serialize};

/// Generate a new invitation token
///
/// This endpoint generates a new invitation token that can be used to invite users to the platform.
#[openapi]
#[post("/token")]
pub async fn generate_invite_token(
    db: &State<Database>,
    user: User,
) -> Result<Json<ApiResponse<InvitationToken>>> {
    // Generate a new invitation token with the user's ID
    let token = db.generate_invite_token(user.id).await?;

    // Return the token directly
    Ok(Json(ApiResponse::ok(token)))
}
