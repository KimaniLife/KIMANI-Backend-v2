use crate::util::iso_bson_chrono;
use serde::{Deserialize, Serialize};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

/// Representation of an invitation token
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvitationToken {
    /// Internal ID
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type of token
    #[serde(rename = "token_type", skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    /// Expiration date of the token
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(
        serialize_with = "iso_bson_chrono::serialize",
        deserialize_with = "iso_bson_chrono::deserialize"
    )]
    pub expiry_date: chrono::DateTime<chrono::Utc>,

    /// The actual token string
    pub token: String,

    /// ID of the user who created this token
    pub creator_id: String,
}
