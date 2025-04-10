#[macro_use]
extern crate rocket;
#[macro_use]
extern crate revolt_rocket_okapi;
#[macro_use]
extern crate serde_json;

pub mod routes;
pub mod util;

use rocket_prometheus::PrometheusMetrics;
use std::net::Ipv4Addr;

use async_std::channel::unbounded;
use revolt_quark::authifier::{Authifier, AuthifierEvent};
use revolt_quark::events::client::EventV1;
use revolt_quark::DatabaseInfo;
use rocket::data::ToByteUnit;
use revolt_database::{Database, MongoDb};

#[launch]
async fn rocket() -> _ {
    // Configure logging and environment
    revolt_quark::configure!();

    // Ensure environment variables are present
    revolt_quark::variables::delta::preflight_checks();

    // --- Setup MongoDb using the public re-export ---
    // Note the change: we use `revolt_database::MongoDb::init(...)` instead of referencing the private drivers module.
    let mongodb = MongoDb::init("mongodb://localhost:27017").await;
    mongodb.migrate_database().await.unwrap();
    
    let database = Database::MongoDb(mongodb);

    // Legacy database setup from quark
    let legacy_db = DatabaseInfo::Auto.connect().await.unwrap();

    // Setup Authifier event channel
    let (sender, receiver) = unbounded();

    // Setup Authifier
    let authifier = Authifier {
        database: legacy_db.clone().into(),
        config: revolt_quark::util::authifier::config(),
        event_channel: Some(sender),
    };

    // Launch a listener for Authifier events
    async_std::task::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match &event {
                AuthifierEvent::CreateSession { .. } | AuthifierEvent::CreateAccount { .. } => {
                    EventV1::Auth(event).global().await
                }
                AuthifierEvent::DeleteSession { user_id, .. }
                | AuthifierEvent::DeleteAllSessions { user_id, .. } => {
                    let id = user_id.to_string();
                    EventV1::Auth(event).private(id).await
                }
            }
        }
    });

    // Launch background task workers
    async_std::task::spawn(revolt_quark::tasks::start_workers(legacy_db.clone()));

    // Configure CORS
    let cors = revolt_quark::web::cors::new();

    // Configure Rocket
    let rocket = rocket::build();
    let prometheus = PrometheusMetrics::new();

    routes::mount(rocket)
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
        .mount("/", revolt_quark::web::cors::catch_all_options_routes())
        .mount("/", revolt_quark::web::ratelimiter::routes())
        .mount("/swagger/", revolt_quark::web::swagger::routes())
        .manage(authifier)
        .manage(database)    // Register the MongoDb instance
        .manage(legacy_db)
        .manage(cors.clone())
        .attach(revolt_quark::web::ratelimiter::RatelimitFairing)
        .attach(cors)
        .configure(rocket::Config {
            limits: rocket::data::Limits::default().limit("string", 5.megabytes()),
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Default::default()
        })
}
