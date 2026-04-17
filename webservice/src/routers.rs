use actix_web::web;
use crate::handlers::{get_courses_for_teacher, health_check_handler, stock_data_handler};
use crate::handlers::new_course;

pub fn general_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}
pub fn course_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/courses"))
        .route("/", web::post().to(new_course))
        .route("/{user_id}", web::get().to(get_courses_for_teacher));
}

pub fn stock_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/stock"))
        .route("/stock_data", web::get().to(stock_data_handler));
}