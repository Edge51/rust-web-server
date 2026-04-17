use actix_web::web;
use serde::{Serialize, Deserialize };
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Option<u32>,
    pub name: String,
    pub teacher_id: u32,
    pub time: Option<chrono::NaiveDateTime>,
}

impl From<web::Json<Course>> for Course {
    fn from(course: web::Json<Course>) -> Self {
        Course {
            id: course.id,
            name: course.name.clone(),
            teacher_id: course.teacher_id,
            time: course.time,
        }
    }
}