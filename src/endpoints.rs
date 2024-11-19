use actix_identity::Identity;
use actix_web::web;
use actix_web::web::Redirect;
use actix_web::HttpResponse;
use actix_web::{get, post, HttpMessage, HttpRequest, Responder};
use chrono::Datelike;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use serde::Deserialize;

use crate::account_data;
use crate::budget_data;

#[get("/")]
pub async fn index(user: Option<Identity>) -> impl Responder {
    let html = std::fs::read_to_string("./templates/index.html").unwrap();
    let message = {
        if let Some(user) = user {
            format!("You are signed in as: {}", user.id().unwrap())
        } else {
            "You are not signed in.".to_owned()
        }
    };
    HttpResponse::Ok().body(html.replace("[[[TEXT]]]", &message))
}

#[derive(Deserialize)]
struct FormInfo {
    name: String,
    password: String,
}

#[post("/do_login")]
pub async fn do_login(
    request: HttpRequest,
    web::Form(form): web::Form<FormInfo>,
) -> impl Responder {
    let result = account_data::get_user_info_sqlite(form.name.clone());
    if result.is_none() {
        return Redirect::to("/login/User%20Not%20Found").see_other();
    }
    if Pbkdf2
        .verify_password(
            form.password.clone().as_bytes(),
            &PasswordHash::new(&result.unwrap().password).unwrap(),
        )
        .is_err()
    {
        return Redirect::to("/login/Password%20Is%20Incorrect").see_other();
    }
    Identity::login(&request.extensions(), form.name.clone()).unwrap();
    Redirect::to("/").see_other()
}
#[get("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok().body(login_html(""))
}
#[get("/login/{message}")]
pub async fn login_message(message: Option<web::Path<String>>) -> impl Responder {
    HttpResponse::Ok().body(login_html(&message.unwrap()))
}
fn login_html(message: &str) -> String {
    let html = std::fs::read_to_string("./templates/login.html").unwrap();
    html.replace("[[[MESSAGE]]]", message)
}

#[post("/do_create_account")]
pub async fn do_create_account(
    request: HttpRequest,
    web::Form(form): web::Form<FormInfo>,
) -> impl Responder {
    let result = account_data::get_user_info_sqlite(form.name.clone());
    if result.is_some() {
        return Redirect::to("/create_account/Username%20already%20exists.").see_other();
    }
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Pbkdf2
        .hash_password(form.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let _ = account_data::insert_user_sqlite(form.name.clone(), password_hash);
    let _ = budget_data::insert_user_sqlite(form.name.clone());
    let _ = budget_data::insert_expendatures(form.name.clone());
    Identity::login(&request.extensions(), form.name.clone()).unwrap();
    Redirect::to("/").see_other()
}
#[get("/create_account")]
pub async fn create_account() -> impl Responder {
    HttpResponse::Ok().body(create_account_html(""))
}
#[get("/create_account/{message}")]
pub async fn create_account_message(message: Option<web::Path<String>>) -> impl Responder {
    HttpResponse::Ok().body(create_account_html(&message.unwrap()))
}
fn create_account_html(message: &str) -> String {
    let html = std::fs::read_to_string("./templates/create_account.html").unwrap();
    html.replace("[[[MESSAGE]]]", message)
}

#[get("/logout")]
pub async fn logout(user: Option<Identity>) -> impl Responder {
    if let Some(user) = user {
        user.logout();
    }
    Redirect::to("/").see_other() // TODO: messaging
}

#[get("/calendar")]
pub async fn calendar(user: Option<Identity>) -> impl Responder {
    if let Some(user) = user {
        return HttpResponse::Ok().body(calendar_html(&user.id().unwrap()));
    }
    HttpResponse::Ok().body(
        r#"<html>Failed.<br><br><a href="/">Return</a></html>"#)
}
fn calendar_html(username: &str) -> String {
    let mut html = std::fs::read_to_string("./templates/calendar.html").unwrap();
    let now = chrono::offset::Local::now();
    let month = chrono::Month::try_from(now.month() as u8 - 1).unwrap().name();
    html = html.replace(
        "[[[CURRENT_MONTH_YEAR]]]",
        &format!("{} {}", &month, &now.year().to_string()),
    );
    html = html.replace(
        "[[[CALENDAR_DIVS]]]",
        &make_calendar_divs(&now, username),
    );
    html = html.replace(
        "[[[TOTAL_TEXT]]]",
        &format!(
            "Net money this month: ${}",
            &budget_data::get_monthly_total(
                username, 
                &chrono::NaiveDate::from_ymd_opt(
                    now.year(),
                    now.month(),
                    now.day(),
                ).unwrap()
            ).to_string()
        ),
    );
    html
}

fn make_calendar_divs(now: &chrono::DateTime<chrono::Local>, username: &str) -> String {
    let mut result = String::new();
    let mut date_iter = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        now.month(),
        1
    ).unwrap();
    let first_day_weekday = 
        chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::new(
                date_iter.clone(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            now.offset().clone(),
        )
        .weekday();
    let mut weekday_iter = chrono::Weekday::Sat;
    while weekday_iter != first_day_weekday {
        weekday_iter = weekday_iter.succ();
        result += &make_calendar_div("", false);
    }
    while date_iter.month0() == now.month0() {
        result += &make_calendar_div(
            &make_calendar_label(username, &date_iter),
            now.day0() == date_iter.day0(),
        );
        date_iter += chrono::Duration::days(1);
    }
    result
}
fn make_calendar_label(username: &str, date: &chrono::NaiveDate) -> String {
    let money = budget_data::get_day_money(username, date);
    let positive = !(money < 0.);
    let money_string = format!(
        r#"<div style="color: {};">{}{}</div>"#,
        if positive { "green" } else { "red" },
        if positive { "$" } else { "-$" },
        money.abs().to_string(),
    );
    format!(
        "<div>{}</div>{}",
        &(date.day0() + 1).to_string(),
        money_string,
    )
}
fn make_calendar_div(text: &str, today: bool) -> String {
    format!(
        r#"<div class="{}">{}</div>"#,
        if today { "today" } else { "day" },
        text,
    )
}

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
    budget_data::add_expendature(
        &user.unwrap().id().unwrap(),
        &budget_data::Expendature::new(
            &chrono::NaiveDate::from_ymd_opt(
                raw_date[0].parse::<i32>().unwrap(),
                raw_date[1].parse::<u32>().unwrap(),
                raw_date[2].parse::<u32>().unwrap(),
            ).unwrap(),
            &form.note,
            form.amount.parse::<f32>().unwrap(),
        )
    );
    Redirect::to("/add_expendature").see_other()
}
#[get("/add_expendature")]
pub async fn add_expendature() -> impl Responder {
    HttpResponse::Ok().body(add_expendature_html())
}
fn add_expendature_html() -> String {
    std::fs::read_to_string("./templates/add_expendature.html").unwrap()
}