use rusqlite::{Connection, params, Result};
use chrono::Datelike;

use crate::data::expendature;

pub fn insert_user_sqlite(username: String) -> Result<()> {
    let mut conn = Connection::open("budgets.db")?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO budget (username, daily, start_day) VALUES (?1, ?2, ?3)",
        [&username, "30", "1"],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn update_daily(username: &str, new_daily: f32) -> Result<()> {
    let conn = Connection::open("budgets.db")?;
    let query = "UPDATE budget SET daily = ?1 WHERE username = ?2";
    conn.execute(query, params![new_daily, username])?;
    Ok(())
}

pub fn update_start_day(username: &str, start_day: u32) -> Result<()> {
    let conn = Connection::open("budgets.db")?;
    let query = "UPDATE budget SET start_day = ?1 WHERE username = ?2";
    conn.execute(query, params![start_day, username])?;
    Ok(())
}



pub fn get_day_money(username: &str, day: &chrono::NaiveDate) -> f32 {
    get_daily(username) - expendature::get_day_expendature(username, day)
}

pub fn get_daily(username: &str) -> f32 {
    let conn = Connection::open("budgets.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT * FROM budget WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([&username]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        //let username: String = row.get(0).unwrap();
        let daily: f32 = row.get(1).unwrap();
        //let start_day: u32 = row.get(2).unwrap();
        return daily;
    }
    6969.
}

pub fn get_start_day(username: &str) -> u32 {
    let conn = Connection::open("budgets.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT * FROM budget WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([&username]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        //let username: String = row.get(0).unwrap();
        //let daily: f32 = row.get(1).unwrap();
        let start_day: u32 = row.get(2).unwrap();
        return start_day;
    }
    6969
}

pub fn get_monthly_total(username: &str, now: &chrono::NaiveDate) -> f32 {
    let start_day = get_start_day(username);
    let within_next_month = now.day() < start_day;
    let start_year = if within_next_month && (now.month() == 0) { now.year() - 1 } else { now.year() };
    let mut start_month = if within_next_month { now.month() - 1 } else { now.month() };
    if start_month <= 0 { 
        start_month = 12
    };
    let mut date_iter = chrono::NaiveDate::from_ymd_opt(start_year, start_month, start_day).unwrap();
    let mut total = 0f32;
    while &date_iter != now {
        total += get_day_money(username, &date_iter);
        date_iter += chrono::Duration::days(1);
    }
    total += get_day_money(username, now);
    total
}