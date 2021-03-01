use serde::Serialize;
use derive_more::{Display, Error};
use warp;

#[derive(Debug, Display, Error, Serialize)]
#[serde(tag="error")]
pub enum ActionError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "not found")]
    NotFound,

    #[display(fmt = "insert error")]
    InsertError,

    #[display(fmt = "invalid form")]
    InvalidForm,

    #[display(fmt = "fetch error")]
    FetchError,
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
