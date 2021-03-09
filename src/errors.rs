use serde::Serialize;
use warp;
use diesel;
use thiserror;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag="error")]
pub enum ActionError {
    #[error("internal error")]
    InternalError,

    #[error("not found")]
    NotFound,

    #[error("insert error")]
    InsertError,

    #[error("invalid form")]
    InvalidForm,

    #[error("fetch error")]
    FetchError,

    #[error("not authenticated")]
    NotAuthenticated,
}

impl warp::reply::Reply for ActionError {
    fn into_response(self) -> warp::reply::Response {
        let code = match &self {
            ActionError::NotFound => warp::http::StatusCode::NOT_FOUND,
            ActionError::InvalidForm => warp::http::StatusCode::UNPROCESSABLE_ENTITY,
            _ => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        warp::reply::with_status(warp::reply::json(&self), code).into_response()
    }
}

pub type ActionResult<T> = Result<T, ActionError>;

impl From<diesel::result::Error> for ActionError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::NotFound => ActionError::NotFound,
            _ => ActionError::InternalError,
        }
    }
}

impl warp::reject::Reject for ActionError {}
