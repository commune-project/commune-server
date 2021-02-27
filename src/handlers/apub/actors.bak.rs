use crate::db::actions;
use crate::state::AppState;
use crate::errors;
use actix_web::{get, web, HttpResponse, Responder};

use crate::apub::serializers::actor::ActorSerializer;

/// extract path info from "/users/{username}" url
/// {username} - deserializes to a String
#[get("/users/{username}")]
pub async fn get_user(
    app_state: web::Data<AppState>,
    (web::Path(username), request): (web::Path<String>, web::HttpRequest),
) -> impl Responder {
    let conn = app_state
        .db
        .get()
        .expect("couldn't get db connection from pool");
    let domain: String = match request.uri().host() {
        Some(s) => String::from(s),
        None => String::from(""),
    };

    let actor = web::block(move || {
        actions::actor::get_account_by_username_domain(&conn, username.as_str(), domain.as_str())
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        match e {
            actix_web::error::BlockingError::Error(diesel::result::Error::NotFound) => HttpResponse::NotFound().json(errors::FetchError::NotFound),
            _ => HttpResponse::InternalServerError().json(errors::FetchError::InternalError)
        }
    });
    match actor {
        Ok(actor) => HttpResponse::Ok().json(ActorSerializer::new(actor).serialize()),
        Err(e) => e,
    }
}
