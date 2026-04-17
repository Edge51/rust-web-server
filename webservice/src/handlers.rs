use std::fs::File;
use actix_web::web;
use actix_web::HttpResponse;
use chrono::Utc;
use crate::models::Course;
use crate::state::AppState;
use csv;
use crate::back_test::data::StockData;

pub async  fn health_check_handler(
    app_state: web::Data<AppState>,
) -> HttpResponse {
    let health_check_response = &app_state.health_check_response;
    let mut visit_count = app_state.visit_count.lock().unwrap();
    let response = format!("{} {} times", health_check_response, visit_count);
    *visit_count += 1;
    HttpResponse::Ok().json(response)
}

pub async fn new_course(
    new_course: web::Json<Course>,
    app_state: web::Data<AppState>,
)-> HttpResponse {
    let course_count = app_state.courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .filter(|c| c.teacher_id == new_course.teacher_id)
        .collect::<Vec<Course>>()
        .len();
    let new_course = Course {
        id: Some((course_count + 1) as u32),
        name: new_course.name.clone(),
        teacher_id: new_course.teacher_id,
        time: Some(Utc::now().naive_utc()),
    };
    app_state.courses.lock().unwrap().push(new_course);
    HttpResponse::Ok().json("New course added")
}
pub async fn stock_data_handler(
) -> HttpResponse {
    println!("Starting stock data");
    let file = File::open("600000_daily_data.csv");
    let mut reader = csv::Reader::from_reader(file.unwrap());
    let headers = "日期,股票代码,开盘,收盘,最高,最低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率\n";
    let mut csv_content = headers.to_string();
    for row in reader.deserialize() {
        let record: StockData
            = row.unwrap();
        let line = format!("{},{},{},{},{},{},{},{},{},{},{},{}\n",
                          record.date,
                          record.code,
                          record.open,
                          record.close.unwrap(),
                          record.high.unwrap(),
                          record.low.unwrap(),
                          record.volume.unwrap(),
                          record.amount.unwrap(),
                          record.amplitude.unwrap(),
                          record.diff.unwrap(),
                          record.diff_ref.unwrap(),
                          record.change.unwrap()
        );
        csv_content.push_str(line.as_str());
    }
    HttpResponse::Ok()
        .content_type("text/txt; charset=utf-8")
        .body(csv_content)

}
pub async fn get_courses_for_teacher(
    app_state: web::Data<AppState>,
    params: web::Path<(u32,)>,
) -> HttpResponse {
    let teacher_id = params.0;
    let filtered_courses :Vec<Course> = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .filter(|course| course.teacher_id == teacher_id)
        .collect();
    if filtered_courses.len() > 0 {
        HttpResponse::Ok().json(filtered_courses)
    } else {
        HttpResponse::Ok().json("No courses founded for this teacher".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test;
    #[actix_rt::test]
    async fn test_health_check_handler() {}

    #[actix_rt::test]
    async fn new_course_test() {
        let course = web::Json(Course {
            id: None,
            teacher_id: 1,
            name: "Test course".into(),
            time: None,
        });
        let app_state = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let resp = new_course(course, app_state).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_courses_for_teacher_test() {
        let app_state = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let teacher_id: web::Path<(u32,)> = web::Path::from((1,));
        let resp = get_courses_for_teacher(app_state, teacher_id).await;
        assert_eq!(resp.status(), StatusCode::OK);

    }
}