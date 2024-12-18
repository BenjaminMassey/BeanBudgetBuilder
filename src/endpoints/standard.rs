use actix_identity::Identity;
use actix_web::web::Redirect;
use actix_web::HttpResponse;
use actix_web::{get, HttpRequest, Responder};

pub async fn error() -> impl Responder {
    HttpResponse::Ok().body(
        std::fs::read_to_string("./templates/error.html").unwrap()
    )
}

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

#[get("/logo.png")]
pub async fn logo(_req_: HttpRequest) -> std::io::Result<actix_files::NamedFile> {
    actix_files::NamedFile::open("./logo.png")
}