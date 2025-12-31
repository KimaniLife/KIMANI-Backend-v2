use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct EventGuest {
    /// Guest ID
    #[serde(rename = "_id")]
    pub id: String,

    /// Event ID this guest belongs to
    pub event_id: String,

    /// Parent guest ID if this is a child guest (for child pricing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_guest_id: Option<String>,

    /// Plus one of this guest ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plus_one_of: Option<String>,

    /// User ID if the guest is a registered user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Guest's full name
    pub name: String,

    /// Guest's email
    pub email: String,

    /// Guest's phone number
    pub phone: String,

    /// Guest status (pending/approved/rejected)
    pub status: GuestStatus,

    /// Is this guest a plus one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_plus_one: Option<bool>,

    /// When the guest was added
    pub created_at: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(into = "String", from = "String")]
pub enum GuestStatus {
    Pending,
    Approved,
    Rejected,
}

impl From<GuestStatus> for String {
    fn from(status: GuestStatus) -> String {
        match status {
            GuestStatus::Pending => "Pending".to_string(),
            GuestStatus::Approved => "Approved".to_string(),
            GuestStatus::Rejected => "Rejected".to_string(),
        }
    }
}

impl From<String> for GuestStatus {
    fn from(status: String) -> Self {
        match status.as_str() {
            "Pending" => GuestStatus::Pending,
            "Approved" => GuestStatus::Approved,
            "Rejected" => GuestStatus::Rejected,
            _ => GuestStatus::Pending, // Default case
        }
    }
}
