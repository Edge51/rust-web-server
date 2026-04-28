mod state;
mod routers;
mod handlers;
mod models;

mod back_test;

use std::sync::Mutex;
use actix_web::{web, App, HttpServer};
use crate::routers::{course_routes, general_routes, stock_routes, backtest_routes};
use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_data = web::Data::new(AppState {
        health_check_response: "I'm OK.".to_string(),
        visit_count: Mutex::new(0),
        courses: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(backtest_routes)
            .configure(stock_routes)
            .configure(general_routes)
            .configure(course_routes)
    })
        .bind("127.0.0.1:8080")?
        .run().await
}

