use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{web, App, HttpServer};
use rusqlite::Connection;

mod data;
mod endpoints;

#[derive(serde::Deserialize, Clone)]
pub struct Server {
    pub address: String,
    pub port: String,
    pub secure: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings: Settings = toml::from_str(
        &std::fs::read_to_string("./settings.toml").unwrap()
    ).expect("Failed to load settings.toml");
    let secret_key = data::account::key_handle();
    db_init();
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(), 
                    secret_key.clone()
                )
                .cookie_secure(settings.server.secure)
                .build(),
            )
            .default_service(web::to(endpoints::standard::error))
            .service(endpoints::standard::index)
            .service(endpoints::standard::landing)
            .service(endpoints::standard::logo)
            .service(endpoints::accounts::login)
            .service(endpoints::accounts::logout)
            .service(endpoints::accounts::do_login)
            .service(endpoints::accounts::create_account)
            .service(endpoints::accounts::do_create_account)
            .service(endpoints::accounts::login_message)
            .service(endpoints::accounts::create_account_message)
            .service(endpoints::calendar::calendar)
            .service(endpoints::calendar::calendar_at_month)
            .service(endpoints::updates::do_add_expendature)
            .service(endpoints::updates::do_update_account)
            .service(endpoints::updates::do_remove_expendature)
    })
    .bind((
        settings.server.address,
        settings.server.port.parse::<u16>().expect("Incorrect port."),
    ))?
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