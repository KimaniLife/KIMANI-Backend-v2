use crate::drivers::mongodb::MongoDb;
use crate::models::trips::model::Trip;
use crate::models::trips::ops::AbstractTrips;
use async_trait::async_trait;
use bson::doc;
use bson::BsonDateTime;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use revolt_result::{create_database_error, Result};

#[async_trait]
impl AbstractTrips for MongoDb {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        let collection = self.col::<Trip>("trips");
        collection
            .insert_one(trip, None)
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("insert", "trips"))
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>> {
        let collection = self.col::<Trip>("trips");
        let mongo_date = bson::DateTime::from_chrono(date);
        let filter = doc! {
            "destination": destination,
            "start_date": { "$lte": mongo_date },
            "end_date": { "$gte": mongo_date }
        };

        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|_| create_database_error!("find", "trips"))?;

        let mut trips = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Ok(trip) = result {
                trips.push(trip);
            }
        }
        Ok(trips)
    }
}
