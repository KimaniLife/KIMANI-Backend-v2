use iso8601_timestamp::Timestamp;
use mongodb::bson::doc;

use crate::models::event::{Event, EventHost, PartialEvent};
use crate::models::saved_event::SavedEvent;
use crate::models::user::User;
use crate::{AbstractEvents, Error, Result};

use super::super::MongoDb;

static COL: &str = "events";
static SAVED_EVENTS_COL: &str = "saved_events";

#[async_trait]
impl AbstractEvents for MongoDb {
    async fn fetch_event(&self, user_id: Option<&str>, id: &str) -> Result<Event> {
        let mut event: Event = self.find_one_by_id(COL, id).await?;

        // Fetch host details
        let host_details = fetch_user_details(self, &event.hosts).await?;
        event.host_details = Some(host_details);

        // Fetch sponsor details
        let sponsor_details = fetch_user_details(self, &event.sponsors).await?;
        event.sponsor_details = Some(sponsor_details);

        // Check saved status
        if let Some(user_id) = user_id {
            event.is_saved = Some(self.is_event_saved(user_id, id).await?);
        }

        Ok(event)
    }
    async fn insert_event(&self, event: &Event) -> Result<()> {
        self.insert_one(COL, event).await.map(|_| ())
    }

    async fn update_event(&self, id: &str, event: &PartialEvent) -> Result<()> {
        self.update_one_by_id(COL, id, event, vec![], None).await?;
        Ok(())
    }

    async fn delete_event(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }

    async fn fetch_events<'a>(
        &self,
        user_id: Option<&str>,
        ids: &'a [String],
    ) -> Result<Vec<Event>> {
        let mut events: Vec<Event> = self.find("events", doc! {}).await?;

        for event in &mut events {
            // Fetch host details
            let host_details = fetch_user_details(self, &event.hosts).await?;
            event.host_details = Some(host_details);

            // Fetch sponsor details
            let sponsor_details = fetch_user_details(self, &event.sponsors).await?;
            event.sponsor_details = Some(sponsor_details);

            // Set saved status
            if let Some(user_id) = user_id {
                event.is_saved = Some(self.is_event_saved(user_id, &event.id).await?);
            } else {
                event.is_saved = None;
            }
        }

        Ok(events)
    }

    async fn toggle_saved_event(&self, user_id: &str, event_id: &str) -> Result<(Event, bool)> {
        let saved_id = format!("{}:{}", user_id, event_id);

        // Fetch the event first
        let event = self.fetch_event(Some(user_id), event_id).await?;

        if event.is_saved.unwrap_or(false) {
            // Unsave - delete the record
            self.delete_one_by_id(SAVED_EVENTS_COL, &saved_id).await?;
            Ok((event, false))
        } else {
            // Save - create new record
            let saved = SavedEvent {
                id: saved_id,
                user_id: user_id.to_string(),
                event_id: event_id.to_string(),
                created_at: Timestamp::now_utc().to_string(),
            };

            self.insert_one(SAVED_EVENTS_COL, &saved).await?;
            Ok((event, true))
        }
    }

    async fn is_event_saved(&self, user_id: &str, event_id: &str) -> Result<bool> {
        let saved_id = format!("{}:{}", user_id, event_id);

        let exists = self
            .col::<SavedEvent>(SAVED_EVENTS_COL)
            .find_one(doc! { "_id": saved_id }, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: SAVED_EVENTS_COL,
            })?;

        Ok(exists.is_some())
    }

    async fn get_saved_events(&self, user_id: &str) -> Result<Vec<Event>> {
        // First get all saved event IDs for this user
        let saved = self
            .find(SAVED_EVENTS_COL, doc! { "user_id": user_id })
            .await?;

        let event_ids: Vec<String> = saved
            .iter()
            .map(|s: &SavedEvent| s.event_id.clone())
            .collect();

        // Then fetch all those events
        if event_ids.is_empty() {
            Ok(vec![])
        } else {
            let mut events: Vec<Event> =
                self.find(COL, doc! { "_id": { "$in": event_ids } }).await?;

            // Set is_saved to true since these are saved events
            for event in &mut events {
                let host_details = fetch_user_details(self, &event.hosts).await?;
                event.host_details = Some(host_details);

                // Fetch sponsor details
                let sponsor_details = fetch_user_details(self, &event.sponsors).await?;
                event.sponsor_details = Some(sponsor_details);
                event.is_saved = Some(true);
            }

            Ok(events)
        }
    }

    async fn get_user_events(&self, user_id: &str) -> Result<Vec<Event>> {
        let mut events: Vec<Event> = self.find(COL, doc! { "created_by": user_id }).await?;

        for event in &mut events {
            // Fetch host details
            let host_details = fetch_user_details(self, &event.hosts).await?;
            event.host_details = Some(host_details);

            // Fetch sponsor details
            let sponsor_details = fetch_user_details(self, &event.sponsors).await?;
            event.sponsor_details = Some(sponsor_details);

            // Set saved status
            event.is_saved = Some(self.is_event_saved(user_id, &event.id).await?);
        }

        Ok(events)
    }
}

// Helper function to fetch user details
async fn fetch_user_details(db: &MongoDb, user_ids: &[String]) -> Result<Vec<EventHost>> {
    let mut hosts = Vec::new();

    for id in user_ids {
        if let Ok(user) = db.find_one::<User>("users", doc! { "_id": id }).await {
            hosts.push(EventHost {
                id: user.id,
                username: user.username,
                avatar: user.avatar,
            });
        }
    }

    Ok(hosts)
}
