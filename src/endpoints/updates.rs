use actix_identity::Identity;
use actix_web::web;
use actix_web::web::Redirect;
use actix_web::{post, HttpRequest, Responder};
use serde::Deserialize;

use crate::data::budget;
use crate::data::expendature;

#[derive(Deserialize)]
struct ExpendatureFormInfo {
    date: String,
    amount: String,
    note: String,
}

#[post("/do_add_expendature")]
pub async fn do_add_expendature(
    user: Option<Identity>,
    _request: HttpRequest,
    web::Form(form): web::Form<ExpendatureFormInfo>,
) -> impl Responder {
    let raw_date: Vec<&str> = form.date.split("-").collect();
    expendature::add_expendature(
        &user.unwrap().id().unwrap(),
        &expendature::Expendature::new(
            &chrono::NaiveDate::from_ymd_opt(
                raw_date[0].parse::<i32>().unwrap(),
                raw_date[1].parse::<u32>().unwrap(),
                raw_date[2].parse::<u32>().unwrap(),
            ).unwrap(),
            &form.note,
            form.amount.parse::<f32>().unwrap(),
        )
    );
    Redirect::to("/").see_other()
}

#[derive(Deserialize)]
struct AccountFormInfo {
    daily: String,
    start: String,
}

#[post("/do_update_account")]
pub async fn do_update_account(
    user: Option<Identity>,
    _request: HttpRequest,
    web::Form(form): web::Form<AccountFormInfo>,
) -> impl Responder {
    if let Ok(num) = form.daily.parse::<f32>() {
        let _ = budget::update_daily(&user.as_ref().unwrap().id().unwrap(), num);
    }
    if let Ok(num) = form.start.parse::<u32>() {
        let _ = budget::update_start_day(&user.as_ref().unwrap().id().unwrap(), num);
    }
    Redirect::to("/").see_other()
}

#[derive(Deserialize)]
struct RemoveFormInfo {
    date: String,
    amount: String,
}

#[post("/do_remove_expendature")]
pub async fn do_remove_expendature(
    user: Option<Identity>,
    web::Form(form): web::Form<RemoveFormInfo>,
) -> impl Responder {
    let date_pieces: Vec<&str> = form.date.split("-").collect();
    let date = chrono::NaiveDate::from_ymd_opt(
        date_pieces[0].parse::<i32>().unwrap(),
        date_pieces[1].parse::<u32>().unwrap(),
        date_pieces[2].parse::<u32>().unwrap(),
    ).unwrap();
    expendature::remove_expendature(
        &user.unwrap().id().unwrap(),
        &date,
        form.amount.parse::<f32>().unwrap(),
    );
    Redirect::to("/").see_other()
}