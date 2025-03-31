use chrono::Utc;
use revolt_quark::models::event::{Event, EventType, TicketConfig};
use revolt_quark::models::user::User;
use revolt_quark::{Database, Error, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataCreateEvent {
    /// Event title
    #[validate(length(min = 1, max = 100))]
    pub title: String,
    /// Event type
    pub event_type: EventType,
    /// Start date and time
    pub start_date: String,
    /// End date and time
    pub end_date: String,
    /// City where event is held
    pub city: String,
    /// Area/neighborhood
    pub area: String,
    /// Full address
    pub address: String,
    /// Event description
    #[validate(length(min = 0, max = 2000))]
    pub description: Option<String>,
    /// Whether to hide the address
    #[serde(default)]
    pub hide_address: bool,
    /// Allow +1 guests
    #[serde(default)]
    pub allow_plus_one: bool,
    /// Maximum number of +1 guests allowed
    #[serde(default)]
    pub allow_plus_one_amount: Option<i32>,
    /// Require full information for +1 guests
    #[serde(default)]
    pub requires_plus_one_info: bool,
    /// Require RSVP approval by host
    #[serde(default)]
    pub requires_rsvp_approval: bool,
    /// Show events to non-members
    #[serde(default)]
    pub show_to_non_members: bool,
    /// Event hosts (user IDs)
    pub hosts: Vec<String>,
    /// Event sponsors (user IDs)
    #[serde(default)]
    pub sponsors: Vec<String>,
    /// Currency type
    #[serde(default)]
    pub currency: Option<String>,
    /// Payment type
    #[serde(default)]
    pub payment_type: Option<String>,
    /// Ticket configuration
    pub ticket_config: TicketConfig,
    /// Attachment URLs
    #[serde(default)]
    pub attachments: Vec<String>,
    /// Gallery image URLs
    #[serde(default)]
    pub gallery: Vec<String>,
    /// Thumbnail image ID
    pub thumbnail: Option<String>,
}

/// Create a new event
#[openapi(tag = "Events")]
#[post("/create", data = "<data>")]
pub async fn create_event(
    db: &State<Database>,
    user: User,
    data: Json<DataCreateEvent>,
) -> Result<Json<Event>> {
    let data = data.into_inner();

    // Validate the input data
    if let Err(validation_errors) = data.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                format!(
                    "{}: {}",
                    field,
                    errors.first().unwrap().message.clone().unwrap_or_default()
                )
            })
            .collect();

        return Err(Error::InvalidRequest {
            code: "validation_error".to_string(),
            errors: error_messages,
        });
    }

    let date = Utc::now().to_rfc3339();
    let event = Event {
        id: Ulid::new().to_string(),
        created_by: Some(user.id),
        title: data.title,
        event_type: Some(data.event_type),
        start_date: data.start_date,
        end_date: data.end_date,
        city: data.city.clone(),
        area: data.area.clone(),
        address: data.address.clone(),
        description: data.description.clone(),
        hide_address: data.hide_address,
        allow_plus_one: data.allow_plus_one,
        allow_plus_one_amount: data.allow_plus_one_amount,
        requires_plus_one_info: data.requires_plus_one_info,
        requires_rsvp_approval: data.requires_rsvp_approval,
        show_to_non_members: data.show_to_non_members,
        hosts: data.hosts.clone(),
        sponsors: data.sponsors.clone(),
        currency: data.currency.clone(),
        payment_type: data.payment_type.clone(),
        ticket_config: data.ticket_config.clone(),
        attachments: data.attachments.clone(),
        gallery: data.gallery.clone(),
        thumbnail: data.thumbnail,
        created_at: date,
        is_saved: None,
        host_details: None,
        sponsor_details: None,
        guests: None,
        guest_stats: None,
    };

    db.insert_event(&event).await?;
    Ok(Json(event))
}
