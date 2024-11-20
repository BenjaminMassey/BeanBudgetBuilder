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
    if user.is_some() {
        return Redirect::to("/calendar").see_other();
    }
    Redirect::to("/landing").see_other()
}

#[get("/landing")]
pub async fn landing() -> impl Responder {
    HttpResponse::Ok().body(
        std::fs::read_to_string("./templates/landing.html").unwrap()
    )
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
    let today = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        now.month(),
        now.day(),
    ).unwrap();
    let daily = budget_data::get_daily(username);
    let start_day = budget_data::get_start_day(username);
    let period_start = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        now.month(),
        start_day,
    ).unwrap();
    let period_end = chrono::NaiveDate::from_ymd_opt(
        if period_start.month0() == 11 { period_start.year() + 1 } else { period_start.year() },
        if period_start.month0() == 11 { 1 } else { period_start.month() + 1 },
        start_day,
    ).unwrap();
    let monthly_money = budget_data::get_monthly_total(
        username, 
        &today
    );
    let positive = monthly_money >= 0.;
    let monthly_money_string = format!(
        r#"<div style="color: {};">{}{}</div>"#,
        if positive { "#95fa92" } else { "#f2a7a7" },
        if positive { "$" } else { "-$" },
        monthly_money.abs().to_string(),
    );
    html = html.replace(
        "[[[HEADER_TEXT]]]",
        &format!(
            "<u>Daily Allotment</u>: ${}<br><br><u>Budget Period</u>: {} to {}<br><br><u>Net money this month</u>: <strong>{}</strong>",
            &daily,
            &format!("{}-{}-{}", period_start.year(), period_start.month(), period_start.day()),
            &format!("{}-{}-{}", period_end.year(), period_end.month(), period_end.day()),
            &monthly_money_string,
        ),
    );
    let mut date_iter = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        now.month(),
        1,
    ).unwrap();
    while date_iter.month() == now.month() {
        html = html.replace(
            &format!("[[[DAY_{}_CONTENT]]]", date_iter.day()),
            &make_popup_content(username, &date_iter),
        );
        date_iter += chrono::Duration::days(1);
    }
    html = html.replace("[[[USERNAME]]]", username);
    html
}

fn make_popup_content(username: &str, date: &chrono::NaiveDate) -> String {
    let mut text = String::new();
    text += &format!(
        "<br><strong>Expendatures for {}-{}-{}</strong><br>",
        date.year(),
        date.month(),
        date.day(),
    );
    for exp in budget_data::get_day_expendatures(username, date) {
        text += &format!("<br>${}: {}", exp.amount, exp.note);
    }
    text += "<br>";
    text
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
    let start_day = budget_data::get_start_day(username);
    let mut weekday_iter = chrono::Weekday::Sat;
    while weekday_iter != first_day_weekday {
        weekday_iter = weekday_iter.succ();
        result += &make_calendar_div("", "day", "blank");
    }
    while date_iter.month0() == now.month0() {
        let is_today = now.day0() == date_iter.day0();
        let is_start = date_iter.day() == start_day;
        result += &make_calendar_div(
            &make_calendar_label(username, &date_iter, is_today),
            if is_today { 
                "today"
            } else if is_start {
                "start-day"
            } else {
                "day"
            },
            &format!("day-{}", date_iter.day()),
        );
        date_iter += chrono::Duration::days(1);
    }
    result
}
fn make_calendar_label(username: &str, date: &chrono::NaiveDate, today: bool) -> String {
    let money = budget_data::get_day_money(username, date);
    let positive = !(money < 0.);
    let day_string = (date.day0() + 1).to_string();
    let money_string = format!(
        r#"<div style="color: {};">{}{}</div>"#,
        if positive { 
            if today { "#95fa92" } else { "green" }
        } else { 
            if today { "#f2a7a7" } else { "red" }
        },
        if positive { "$" } else { "-$" },
        money.abs().to_string(),
    );
    format!(
        "<div>{}</div>{}",
        &day_string,
        money_string,
    )
}
fn make_calendar_div(text: &str, class: &str, id: &str) -> String {
    format!(r#"<div id="{id}" class="{class}">{text}</div>"#)
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
        let _ = budget_data::update_daily(&user.as_ref().unwrap().id().unwrap(), num);
    }
    if let Ok(num) = form.start.parse::<u32>() {
        let _ = budget_data::update_start_day(&user.as_ref().unwrap().id().unwrap(), num);
    }
    Redirect::to("/update_account").see_other()
}
#[get("/update_account")]
pub async fn update_account() -> impl Responder {
    HttpResponse::Ok().body(update_account_html())
}
fn update_account_html() -> String {
    std::fs::read_to_string("./templates/update_account.html").unwrap()
}