use std::sync::Arc;

use rocket::{launch, routes};

use crate::background::BackgroundWorkers;
use crate::state::AppState;

mod background;
mod metrics;
mod registry;
mod routes;
mod state;
mod storage;
mod styles;
mod views;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Arc::new(AppState::new()))
        .attach(BackgroundWorkers)
        .mount(
            "/",
            routes![
                routes::heartbeat,
                routes::record_metric,
                views::styles,
                views::index,
                views::bot_detail::bot_detail,
                views::bot_list::fragment_bot_list,
                views::bot_detail::fragment_bot_charts,
            ],
        )
}
