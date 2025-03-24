use crate::models::event::{Event, PartialEvent};
use crate::models::events::guest::{EventGuest, GuestStatus};
use crate::models::user::User;
use crate::Result;

#[async_trait]
pub trait AbstractEvents: Sync + Send {
    async fn fetch_event(&self, user_id: Option<&str>, id: &str) -> Result<Event>;
    async fn fetch_events<'a>(
        &self,
        user_id: Option<&str>,
        ids: &'a [String],
    ) -> Result<Vec<Event>>;
    async fn insert_event(&self, event: &Event) -> Result<()>;
    async fn update_event(&self, id: &str, event: &PartialEvent) -> Result<()>;
    async fn delete_event(&self, id: &str) -> Result<()>;
    async fn toggle_saved_event(&self, user_id: &str, event_id: &str) -> Result<(Event, bool)>;
    async fn is_event_saved(&self, user_id: &str, event_id: &str) -> Result<bool>;
    async fn get_saved_events(&self, user_id: &str) -> Result<Vec<Event>>;
    /// Get all events created by a user
    async fn get_user_events(&self, user_id: &str) -> Result<Vec<Event>>;
    /// Add a guest to an event
    async fn add_guest(&self, guest: &EventGuest) -> Result<()>;
    /// Update guest status
    async fn update_guest_status(
        &self,
        event_id: &str,
        guest_id: &str,
        status: GuestStatus,
    ) -> Result<()>;
    /// Get event guests
    async fn get_event_guests(&self, event_id: &str) -> Result<Vec<EventGuest>>;
    /// Get guest by ID
    async fn get_guest(&self, event_id: &str, guest_id: &str) -> Result<()>;
}
