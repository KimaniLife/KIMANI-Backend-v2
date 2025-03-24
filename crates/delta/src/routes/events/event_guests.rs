use revolt_quark::models::events::guest::{EventGuest, GuestStatus};
use revolt_quark::models::user::User;
use revolt_quark::{Database, Error, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataCreateGuest {
    /// Guest's full name
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// Guest's email
    #[validate(email)]
    pub email: String,
    /// Guest's phone number
    pub phone: String,
    /// Associated user ID if the guest is a registered user
    pub associated_user_id: Option<String>,
    /// If this guest is a plus one of another guest
    pub plus_one_of: Option<String>,
    /// If this guest is a child guest (for pricing)
    pub parent_guest_id: Option<String>,
}

#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataCreateBulkGuests {
    /// Main contact guest
    pub main_contact: DataCreateGuest,
    /// Additional guests
    pub additional_guests: Vec<DataCreateGuest>,
}

#[derive(Serialize, JsonSchema)]
pub struct GuestResponse {
    guest: EventGuest,
}

#[derive(Serialize, JsonSchema)]
pub struct BulkGuestResponse {
    guests: Vec<EventGuest>,
}

#[derive(Serialize, JsonSchema)]
pub struct UserSearchResponse {
    id: String,
    username: String,
    email: String,
}

#[derive(Validate, Deserialize, JsonSchema)]
pub struct GuestStatusUpdate {
    /// Guest ID
    pub guest_id: String,
    /// Status to set
    pub status: GuestStatus,
}

#[derive(Validate, Deserialize, JsonSchema)]
pub struct BulkStatusUpdate {
    /// List of guest status updates
    pub updates: Vec<GuestStatusUpdate>,
}

/// Add a guest to an event
#[openapi(tag = "Events")]
#[post("/<event_id>/guests", data = "<data>")]
pub async fn add_guest(
    db: &State<Database>,
    user: User,
    event_id: String,
    data: Json<DataCreateGuest>,
) -> Result<Json<GuestResponse>> {
    let data = data.into_inner();

    // Validate the input data
    if let Err(validation_errors) = data.validate() {
        return Err(Error::InvalidRequest {
            code: "validation_error".to_string(),
            errors: validation_errors_to_strings(&validation_errors),
        });
    }
    let guest = EventGuest {
        id: Ulid::new().to_string(),
        event_id: event_id.clone(),
        name: data.name,
        email: data.email,
        phone: data.phone,
        status: GuestStatus::Pending,
        user_id: Some(user.id.clone()),
        plus_one_of: data.plus_one_of.clone(),
        parent_guest_id: data.parent_guest_id,
        is_plus_one: Some(data.plus_one_of.is_some()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db.add_guest(&guest).await?;
    Ok(Json(GuestResponse { guest }))
}

/// Update single guest status
#[openapi(tag = "Events")]
#[patch("/<event_id>/guests/<guest_id>/status", data = "<status>")]
pub async fn update_guest_status(
    db: &State<Database>,
    _user: User,
    event_id: String,
    guest_id: String,
    status: Json<GuestStatus>,
) -> Result<Json<()>> {
    db.update_guest_status(&event_id, &guest_id, status.into_inner())
        .await?;
    Ok(Json(()))
}

/// Update multiple guests' status
#[openapi(tag = "Events")]
#[patch("/<event_id>/guests/bulk/status", data = "<data>")]
pub async fn update_bulk_guest_status(
    db: &State<Database>,
    _user: User,
    event_id: String,
    data: Json<BulkStatusUpdate>,
) -> Result<Json<()>> {
    let data = data.into_inner();
    for update in data.updates {
        db.update_guest_status(&event_id, &update.guest_id, update.status)
            .await?;
    }
    Ok(Json(()))
}

/// Get all guests for an event
#[openapi(tag = "Events")]
#[get("/<event_id>/guests")]
pub async fn get_event_guests(
    db: &State<Database>,
    _user: User,
    event_id: String,
) -> Result<Json<Vec<EventGuest>>> {
    let guests = db.get_event_guests(&event_id).await?;
    Ok(Json(guests))
}

/// Get a specific guest
#[openapi(tag = "Events")]
#[get("/<event_id>/guests/<guest_id>")]
pub async fn get_guest(
    db: &State<Database>,
    _user: User,
    event_id: String,
    guest_id: String,
) -> Result<Json<()>> {
    let guest = db.get_guest(&event_id, &guest_id).await?;
    Ok(Json(guest))
}

/// Add multiple guests to an event
#[openapi(tag = "Events")]
#[post("/<event_id>/guests/bulk", data = "<data>")]
pub async fn add_bulk_guests(
    db: &State<Database>,
    user: Option<User>,
    event_id: String,
    data: Json<DataCreateBulkGuests>,
) -> Result<Json<BulkGuestResponse>> {
    let data = data.into_inner();
    let mut created_guests = Vec::new();

    // First create the main contact
    if let Err(validation_errors) = data.main_contact.validate() {
        return Err(Error::InvalidRequest {
            code: "validation_error".to_string(),
            errors: validation_errors_to_strings(&validation_errors),
        });
    }

    let main_guest = EventGuest {
        id: Ulid::new().to_string(),
        event_id: event_id.clone(),
        name: data.main_contact.name,
        email: data.main_contact.email,
        phone: data.main_contact.phone,
        status: GuestStatus::Pending,
        user_id: user.map(|u| u.id),
        plus_one_of: None,
        parent_guest_id: None,
        is_plus_one: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db.add_guest(&main_guest).await?;
    created_guests.push(main_guest.clone());

    // Then create additional guests
    for guest_data in data.additional_guests {
        if let Err(validation_errors) = guest_data.validate() {
            return Err(Error::InvalidRequest {
                code: "validation_error".to_string(),
                errors: validation_errors_to_strings(&validation_errors),
            });
        }

        let guest = EventGuest {
            id: Ulid::new().to_string(),
            event_id: event_id.clone(),
            name: guest_data.name,
            email: guest_data.email,
            phone: guest_data.phone,
            status: GuestStatus::Pending,
            user_id: None,
            plus_one_of: Some(main_guest.id.clone()),
            parent_guest_id: guest_data.parent_guest_id,
            is_plus_one: Some(true),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.add_guest(&guest).await?;
        created_guests.push(guest);
    }

    Ok(Json(BulkGuestResponse {
        guests: created_guests,
    }))
}

// Helper function to convert validation errors to strings
fn validation_errors_to_strings(errors: &validator::ValidationErrors) -> Vec<String> {
    errors
        .field_errors()
        .iter()
        .map(|(field, errors)| {
            format!(
                "{}: {}",
                field,
                errors.first().unwrap().message.clone().unwrap_or_default()
            )
        })
        .collect()
}
