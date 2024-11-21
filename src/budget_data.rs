use rusqlite::{Connection, params, Result};
use chrono::Datelike;

const MAJOR_SPLIT: &str = "<!>";
const MINOR_SPLIT: &str = ":!:";

pub struct Expendature {
    pub date: chrono::NaiveDate,
    pub amount: f32,
    pub note: String,
}
impl Expendature {
    pub fn new(date: &chrono::NaiveDate, note: &str, amount: f32) -> Self {
        Self {
            date: date.clone(), 
            note: note.to_owned(),
            amount
        }
    }
    pub fn string(self: &Self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            self.date.year(),
            "-",
            self.date.month(),
            "-",
            self.date.day(),
            MINOR_SPLIT,
            self.amount,
            MINOR_SPLIT,
            self.note,
            MAJOR_SPLIT,
        )
    }
}

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

pub fn insert_expendatures(username: String) -> Result<()> {
    let mut conn = Connection::open("budgets.db")?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO expendatures (username, data) VALUES (?1, ?2)",
        [&username, ""],
    )?;
    tx.commit()?;
    Ok(())
}

fn update_expendatures_data(username: &str, data: &str) -> Result<()> {
    let conn = Connection::open("budgets.db")?;
    let query = "UPDATE expendatures SET data = ?1 WHERE username = ?2";
    conn.execute(query, params![data, username])?;
    Ok(()) 
}

pub fn get_day_money(username: &str, day: &chrono::NaiveDate) -> f32 {
    get_daily(username) - get_day_expendature(username, day)
}

pub fn get_day_expendature(
    username: &str,
    date: &chrono::NaiveDate
) -> f32 {
    let mut total = 0f32;
    for expendature in get_expendatures(username) {
        if &expendature.date == date {
            total += expendature.amount;
        }
    }
    total
}

pub fn get_expendatures(username: &str) -> Vec<Expendature> {
    let conn = Connection::open("budgets.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT * FROM expendatures WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([&username]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        //let username: String = row.get(0).unwrap();
        let data: String = row.get(1).unwrap();
        let mut expendatures: Vec<Expendature> = vec![];
        for item in data.split(MAJOR_SPLIT) {
            if item.is_empty() {
                continue;
            }
            let info: Vec<&str> = item.split(MINOR_SPLIT).collect();
            let raw_date: Vec<&str> = info[0].split("-").collect();
            let date = chrono::NaiveDate::from_ymd_opt(
                raw_date[0].parse::<i32>().unwrap(),
                raw_date[1].parse::<u32>().unwrap(),
                raw_date[2].parse::<u32>().unwrap(),
            ).unwrap();
            expendatures.push(
                Expendature {
                    date,
                    amount: info[1].parse::<f32>().unwrap(),
                    note: info[2].to_owned(),
                }
            );
        }
        return expendatures;
    }
    vec![]
}


pub fn get_day_expendatures(
    username: &str, 
    date: &chrono::NaiveDate,
) -> Vec<Expendature> {
    let mut expendatures: Vec<Expendature> = vec![];
    for exp in get_expendatures(username) {
        if &exp.date == date {
            expendatures.push(exp);
        }
    }
    expendatures
}

pub fn add_expendature(username: &str, expendature: &Expendature) {
    let mut data = String::new();
    for x in get_expendatures(username) {
        data += &x.string();
    }
    data += &expendature.string();
    let _ = update_expendatures_data(username, &data);
}

pub fn remove_expendature(username: &str, date: &chrono::NaiveDate, amount: f32) {
    let mut data = String::new();
    for x in get_expendatures(username) {
        if !(&x.date == date && x.amount == amount) {
            data += &x.string();
        }
    }
    let _ = update_expendatures_data(username, &data);
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