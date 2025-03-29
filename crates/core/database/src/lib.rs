#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_recursion;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

#[macro_use]
extern crate revolt_optional_struct;

#[macro_use]
extern crate revolt_result;

#[cfg(feature = "mongodb")]
pub use ::mongodb;
//pub use drivers::mongodb;

use futures::stream::StreamExt;

#[cfg(feature = "mongodb")]
#[macro_use]
extern crate bson;

use crate::models::invite_user::model::InvitationToken;
use crate::models::invite_user::AbstractInviteTokens;
use crate::models::trips::model::{Trip, TripComment};

macro_rules! database_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Clone)]
            $item
        )+
    };
}

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
            $item
        )+
    };
}

macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(OptionalStruct, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
        #[optional_derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

// ----------------------------------------------------------------------------
// Modules
// ----------------------------------------------------------------------------

pub mod drivers;
pub use drivers::*;

#[cfg(test)]
macro_rules! database_test {
    ( | $db: ident | $test:expr ) => {
        let db = $crate::DatabaseInfo::Test(format!(
            "{}:{}",
            file!().replace('/', "_").replace(".rs", ""),
            line!()
        ))
        .connect()
        .await
        .expect("Database connection failed.");

        db.drop_database().await;

        #[allow(clippy::redundant_closure_call)]
        (|$db: $crate::Database| $test)(db.clone()).await;

        db.drop_database().await
    };
}

mod models;
pub mod util;
pub use models::*;

pub mod events;

// ----------------------------------------------------------------------------
// Utility
// ----------------------------------------------------------------------------

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

// ----------------------------------------------------------------------------
// DatabaseTrait setup
// ----------------------------------------------------------------------------

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use revolt_result::{create_database_error, Result};

pub use crate::drivers::Database;
pub use drivers::mongodb::MongoDb;

use bson::{doc, oid::ObjectId, DateTime as BsonDateTime};

/// A trait exposing the DB methods your code calls
#[async_trait]
pub trait DatabaseTrait: Sync + Send {
    async fn insert_trip(&self, trip: &Trip) -> Result<()>;
    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
        current_user_id: &str,
    ) -> Result<Vec<Trip>>;
    /// Marks all trips from a user in a destination as deleted, except for a specific trip
    async fn mark_user_trips_as_deleted_in_destination(
        &self,
        user_id: &str,
        destination: &str,
        except_trip_id: Option<ObjectId>,
    ) -> Result<()>;
    /// Marks a specific trip as deleted
    async fn delete_trip(&self, trip_id: ObjectId, user_id: &str) -> Result<()>;
    /// Creates a new comment on a trip
    async fn create_trip_comment(&self, comment: &TripComment) -> Result<()>;
    /// Fetches all comments for a trip in a destination
    async fn fetch_trip_comments_by_destination(
        &self,
        trip_id: ObjectId,
        destination: &str,
    ) -> Result<Vec<TripComment>>;
    /// Generate a new invitation token
    async fn generate_invite_token(&self, creator_id: String) -> Result<InvitationToken>;
}

// ----------------------------------------------------------------------------
// Implementation for MongoDb
// ----------------------------------------------------------------------------

#[async_trait]
impl DatabaseTrait for MongoDb {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        let mut new_trip = trip.clone();
        let id = ObjectId::new();
        new_trip.id = Some(id);

        let collection = self.col::<Trip>("trips");
        match collection.insert_one(&new_trip, None).await {
            Ok(_res) => {
                self.mark_user_trips_as_deleted_in_destination(
                    &trip.user_id,
                    &trip.destination,
                    Some(id),
                )
                .await?;

                Ok(())
            }
            Err(_) => Err(create_database_error!("insert", "trips")),
        }
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
        current_user_id: &str,
    ) -> Result<Vec<Trip>> {
        let today = Utc::now().date().and_hms(0, 0, 0);
        let today_bson = BsonDateTime::from_chrono(today);

        let collection = self.col::<Trip>("trips");

        let pipeline = vec![
            doc! {
                "$match": {
                    "destination": destination,
                    "start_date": { "$gte": today_bson },
                    "$or": [
                        { "deletion_date": { "$exists": false } },
                        { "deletion_date": null }
                    ]
                }
            },
            doc! {
                "$addFields": {
                    "sortOrder": {
                        "$switch": {
                            "branches": [
                                {
                                    "case": { "$eq": ["$user_id", current_user_id] },
                                    "then": 0
                                }
                            ],
                            "default": 1
                        }
                    }
                }
            },
            doc! {
                "$sort": {
                    "sortOrder": 1,
                    "end_date": 1
                }
            },
            doc! {
                "$project": {
                    "sortOrder": 0
                }
            },
        ];

        let mut cursor = collection
            .aggregate(pipeline, None)
            .await
            .map_err(|_| create_database_error!("find", "trips"))?;

        let mut trips = Vec::new();

        while let Some(doc) = cursor.next().await {
            if let Ok(doc) = doc {
                if let Ok(trip) = bson::from_document(doc) {
                    trips.push(trip);
                }
            }
        }

        Ok(trips)
    }

    async fn mark_user_trips_as_deleted_in_destination(
        &self,
        user_id: &str,
        destination: &str,
        except_trip_id: Option<ObjectId>,
    ) -> Result<()> {
        let collection = self.col::<Trip>("trips");

        // Get current time in UTC
        let now = BsonDateTime::now();

        // Build filter to match only future trips:
        let mut filter = doc! {
            "user_id": user_id,
            "destination": destination,
            "start_date": { "$gt": now },  // Only mark future trips as deleted
            "deletion_date": { "$exists": false }
        };

        // Add except_trip_id to filter if provided
        if let Some(trip_id) = except_trip_id {
            filter.insert("_id", doc! { "$ne": trip_id });
        }

        // Update document - set deletion_date to current time
        let update = doc! {
            "$set": {
                "deletion_date": now
            }
        };

        match collection.update_many(filter, update, None).await {
            Ok(_) => Ok(()),
            Err(err) => Err(create_database_error!("update", "trips")),
        }
    }

    async fn delete_trip(&self, trip_id: ObjectId, user_id: &str) -> Result<()> {
        let collection = self.col::<Trip>("trips");

        // Get current time in UTC
        let now = BsonDateTime::now();

        let filter = doc! {
            "_id": trip_id,
            "user_id": user_id,  // Ensure user owns the trip
            "$or": [  // $or needs to be at the top level of the query
                { "deletion_date": { "$exists": false } },
                { "deletion_date": null }
            ]  // Only delete if not already deleted
        };

        let update = doc! {
            "$set": {
                "deletion_date": now
            }
        };

        match collection.update_one(filter, update, None).await {
            Ok(result) => {
                if result.modified_count == 0 {
                    // Check if the trip exists at all
                    let trip_exists = collection
                        .find_one(doc! { "_id": trip_id }, None)
                        .await
                        .map_err(|_| create_database_error!("find", "trips"))?;

                    match trip_exists {
                        Some(_) => Err(create_database_error!("unauthorized", "trip")), // Trip exists but user doesn't own it
                        None => Err(create_database_error!("not_found", "trip")), // Trip doesn't exist
                    }
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(create_database_error!("update", "trips")),
        }
    }

    async fn create_trip_comment(&self, comment: &TripComment) -> Result<()> {
        let mut new_comment = comment.clone();
        new_comment.id = Some(ObjectId::new());
        new_comment.created_at = Utc::now();

        let collection = self.col::<TripComment>("trip_comments");
        match collection.insert_one(&new_comment, None).await {
            Ok(_) => Ok(()),
            Err(err) => Err(create_database_error!("insert", "trip_comments")),
        }
    }

    async fn fetch_trip_comments_by_destination(
        &self,
        trip_id: ObjectId,
        destination: &str,
    ) -> Result<Vec<TripComment>> {
        let trips_collection = self.col::<Trip>("trips");
        let comments_collection = self.col::<TripComment>("trip_comments");

        // First verify the trip exists and matches the destination
        let trip = trips_collection
            .find_one(
                doc! {
                    "_id": trip_id,
                    "destination": destination,
                    "$or": [
                        { "deletion_date": { "$exists": false } },
                        { "deletion_date": null }
                    ]
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", "trips"))?;

        // If trip doesn't exist or doesn't match destination, return error
        if trip.is_none() {
            return Err(create_database_error!("not_found", "trip"));
        }

        // Get comments for this trip
        let pipeline = vec![
            // Match comments for the specific trip
            doc! {
                "$match": {
                    "trip_id": trip_id
                }
            },
            // Sort by created_at descending (newest first)
            doc! {
                "$sort": {
                    "created_at": -1
                }
            },
        ];

        let mut cursor = comments_collection
            .aggregate(pipeline, None)
            .await
            .map_err(|_| create_database_error!("aggregate", "trip_comments"))?;

        let mut comments = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => {
                    if let Ok(comment) = bson::from_document(doc) {
                        comments.push(comment);
                    }
                }
                Err(_) => return Err(create_database_error!("find", "trip_comments")),
            }
        }

        Ok(comments)
    }

    /// Generate a new invitation token
    async fn generate_invite_token(&self, creator_id: String) -> Result<InvitationToken> {
        AbstractInviteTokens::generate_invite_token(self, creator_id).await
    }
}

// ----------------------------------------------------------------------------
// Implementation for enum Database
// ----------------------------------------------------------------------------

#[async_trait]
impl DatabaseTrait for Database {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        match self {
            Database::MongoDb(mongo) => mongo.insert_trip(trip).await,
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for insert_trip.")
            }
        }
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
        current_user_id: &str,
    ) -> Result<Vec<Trip>> {
        match self {
            Database::MongoDb(mongo) => {
                mongo
                    .fetch_trips_by_date_and_destination(date, destination, current_user_id)
                    .await
            }
            Database::Reference(_mock) => {
                unimplemented!(
                    "Reference DB not implemented for fetch_trips_by_date_and_destination."
                )
            }
        }
    }

    async fn mark_user_trips_as_deleted_in_destination(
        &self,
        user_id: &str,
        destination: &str,
        except_trip_id: Option<ObjectId>,
    ) -> Result<()> {
        match self {
            Database::MongoDb(mongo) => {
                mongo
                    .mark_user_trips_as_deleted_in_destination(user_id, destination, except_trip_id)
                    .await
            }
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for mark_user_trips_as_deleted.")
            }
        }
    }

    async fn delete_trip(&self, trip_id: ObjectId, user_id: &str) -> Result<()> {
        match self {
            Database::MongoDb(mongo) => mongo.delete_trip(trip_id, user_id).await,
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for delete_trip.")
            }
        }
    }

    async fn create_trip_comment(&self, comment: &TripComment) -> Result<()> {
        match self {
            Database::MongoDb(mongo) => mongo.create_trip_comment(comment).await,
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for create_trip_comment.")
            }
        }
    }

    async fn fetch_trip_comments_by_destination(
        &self,
        trip_id: ObjectId,
        destination: &str,
    ) -> Result<Vec<TripComment>> {
        match self {
            Database::MongoDb(mongo) => {
                mongo
                    .fetch_trip_comments_by_destination(trip_id, destination)
                    .await
            }
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for fetch_trip_comments.")
            }
        }
    }

    /// Generate a new invitation token
    async fn generate_invite_token(&self, creator_id: String) -> Result<InvitationToken> {
        match self {
            Database::MongoDb(mongo) => mongo.generate_invite_token(creator_id).await,
            Database::Reference(_mock) => {
                unimplemented!("Reference DB not implemented for generate_invite_token.")
            }
        }
    }
}
