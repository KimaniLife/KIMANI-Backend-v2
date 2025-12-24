use revolt_rocket_okapi::openapi_get_routes_spec;
use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod open_dm;

pub use open_dm::*;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![req]
}
