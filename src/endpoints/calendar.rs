use actix_identity::Identity;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::{get, Responder};
use chrono::Datelike;

use crate::data::budget;
use crate::data::expendature;

#[get("/calendar")]
pub async fn calendar(user: Option<Identity>) -> impl Responder {
    if let Some(user) = user {
        return HttpResponse::Ok().body(calendar_html(&user.id().unwrap(), None));
    }
    HttpResponse::Ok().body(
        std::fs::read_to_string("./templates/error.html").unwrap()
    )
}
struct CalendarParams {
    year: u32,
    month: u32,
}
#[get("/calendar/{date_param}")]
pub async fn calendar_at_month(
    user: Option<Identity>,
    date_param: Option<web::Path<String>>,
) -> impl Responder {
    let date_string = &date_param.unwrap();
    let date_pieces: Vec<&str> = date_string.split("-").collect();
    let params = CalendarParams {
        year: date_pieces[0].parse::<u32>().unwrap(),
        month: date_pieces[1].parse::<u32>().unwrap(),
    };
    if let Some(user) = user {
        return HttpResponse::Ok().body(calendar_html(&user.id().unwrap(), Some(params)));
    }
    HttpResponse::Ok().body(
        std::fs::read_to_string("./templates/error.html").unwrap()
    )
}
fn calendar_html(username: &str, params: Option<CalendarParams>) -> String {
    let mut html = std::fs::read_to_string("./templates/calendar.html").unwrap();
    let now = chrono::offset::Local::now();
    let cal_year = if let Some(params) = params.as_ref() {
        params.year as i32
    } else {
        now.year()
    };
    let cal_month = if let Some(params) = params.as_ref() {
        params.month
    } else {
        now.month()
    };
    let month = chrono::Month::try_from(cal_month as u8).unwrap().name();
    html = html.replace(
        "[[[CURRENT_MONTH_YEAR]]]",
        &format!("{} {}", &month, &cal_year.to_string()),
    );
    html = html.replace(
        "[[[PREV_MONTH_LINK]]]",
        &format!(
            "{}-{}",
            if cal_month == 1 { cal_year - 1 } else { cal_year },
            if cal_month == 1 { 12 } else { cal_month - 1 },
        ),
    );
    html = html.replace(
        "[[[NEXT_MONTH_LINK]]]",
        &format!(
            "{}-{}",
            if cal_month == 12 { cal_year + 1 } else { cal_year },
            if cal_month == 12 { 1 } else { cal_month + 1 },
        ),
    );
    html = html.replace(
        "[[[CALENDAR_DIVS]]]",
        &make_calendar_divs(
            username,
            cal_year,
            cal_month,
            if cal_month == now.month() { Some(now.day0()) } else { None },
            now.offset(),
        ),
    );
    let today = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        now.month(),
        now.day(),
    ).unwrap();
    let daily = budget::get_daily(username);
    let start_day = budget::get_start_day(username);
    let period_start = chrono::NaiveDate::from_ymd_opt(
        now.year(),
        if start_day <= now.day() {
            now.month() 
        } else {
            if cal_month == 1 { 12 } else { now.month() - 1 }
        },
        start_day,
    ).unwrap();
    let period_end = chrono::NaiveDate::from_ymd_opt(
        if period_start.month0() == 11 { period_start.year() + 1 } else { period_start.year() },
        if period_start.month0() == 11 { 1 } else { period_start.month() + 1 },
        start_day,
    ).unwrap();
    let monthly_money = budget::get_monthly_total(
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
        cal_year,
        cal_month,
        1,
    ).unwrap();
    while date_iter.month() == cal_month {
        html = html.replace(
            &format!("[[[DAY_{}_CONTENT]]]", date_iter.day()),
            &make_popup_content(username, &date_iter),
        );
        date_iter += chrono::Duration::days(1);
    }
    html = html.replace("[[[USERNAME]]]", username);
    html = html.replace("[[[CURRENT_YEAR]]]", &cal_year.to_string());
    html = html.replace("[[[CURRENT_MONTH]]]", &cal_month.to_string());
    html
}

fn make_popup_content(username: &str, date: &chrono::NaiveDate) -> String {
    let mut text = String::new();
    let date_string = format!(
        "{}-{}-{}",
        date.year(),
        date.month(),
        date.day(),
    );
    text += &format!("<br><strong>Expendatures for {date_string}</strong><br>");
    for exp in expendature::get_day_expendatures(username, date) {
        text += &format!(
            r#"<br><div class="expendature-item">${}: {}"#,
            exp.amount,
            exp.note,
        );
        text += std::fs::read_to_string("./templates/component_remove_expendature.html")
            .unwrap()
            .replace("[[[DATE]]]", &date_string)
            .replace("[[[AMOUNT]]]", &exp.amount.to_string())
            .as_ref();
        text += "</div>";
    }
    text += "<br>";
    text
}

fn make_calendar_divs(
    username: &str,
    year: i32,
    month: u32,
    day: Option<u32>,
    offset: &chrono::FixedOffset,
) -> String {
    let mut result = String::new();
    let mut date_iter = chrono::NaiveDate::from_ymd_opt(
        year,
        month,
        1
    ).unwrap();
    let first_day_weekday = 
        chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::new(
                date_iter.clone(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            offset.clone(),
        )
        .weekday();
    let start_day = budget::get_start_day(username);
    let mut weekday_iter = chrono::Weekday::Sat;
    while weekday_iter != first_day_weekday {
        weekday_iter = weekday_iter.succ();
        result += &make_calendar_div("", "blank-day", "blank");
    }
    while date_iter.month0() == month - 1 {
        let is_today = if let Some(day) = day { date_iter.day0() == day } else { false };
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
    let money = budget::get_day_money(username, date);
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