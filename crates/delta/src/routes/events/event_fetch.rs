use revolt_quark::models::event::EventGuestStats;
use revolt_quark::models::events::guest::{EventGuest, GuestStatus};
use revolt_quark::{models::event::Event, models::user::User, Database, Result};
use rocket::{serde::json::Json, State};

/// Get event by id
#[openapi(tag = "Events")]
#[get("/<id>")]
pub async fn get_event(
    db: &State<Database>,
    user: Option<User>,
    id: String,
) -> Result<Json<Event>> {
    let mut event = db
        .fetch_event(user.as_ref().map(|u| u.id.as_str()), &id)
        .await?;

    // Calculate guest statistics
    let guests = db.get_event_guests(&id).await?;
    let stats = EventGuestStats {
        total_invited: guests.len() as i32,
        total_going: guests
            .iter()
            .filter(|g| matches!(g.status, GuestStatus::Approved))
            .count() as i32,
        total_pending: guests
            .iter()
            .filter(|g| matches!(g.status, GuestStatus::Pending))
            .count() as i32,
        total_rejected: guests
            .iter()
            .filter(|g| matches!(g.status, GuestStatus::Rejected))
            .count() as i32,
    };

    event.guest_stats = Some(stats);
    Ok(Json(event))
}
