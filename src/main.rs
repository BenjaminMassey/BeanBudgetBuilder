use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{App, HttpServer};
use rusqlite::Connection;

mod account_data;
mod endpoints;
mod budget_data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let secret_key = account_data::key_handle();
    db_init();
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(endpoints::index)
            .service(endpoints::login)
            .service(endpoints::logout)
            .service(endpoints::do_login)
            .service(endpoints::create_account)
            .service(endpoints::do_create_account)
            .service(endpoints::login_message)
            .service(endpoints::create_account_message)
            .service(endpoints::calendar)
            .service(endpoints::do_add_expendature)
            .service(endpoints::add_expendature)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn db_init() {
    let accounts = Connection::open("accounts.db").unwrap();
    accounts.execute(
        "CREATE TABLE IF NOT EXISTS users (
             username TEXT PRIMARY KEY,
             password TEXT)",
        [],
    )
    .unwrap();
    let budgets = Connection::open("budgets.db").unwrap();
    budgets.execute(
        "CREATE TABLE IF NOT EXISTS budget (
            username TEXT PRIMARY KEY,
            daily INTEGER,
            start_day INTEGER)",
        [],
    )
    .unwrap();
    budgets.execute(
        "CREATE TABLE IF NOT EXISTS expendatures (
            username TEXT PRIMARY KEY,
            data TEXT)",
        [],
    )
    .unwrap();
}