use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};

use crate::errors::DisplexError;

// Tell axum how to convert `DisplexError` into a response.
impl IntoResponse for DisplexError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
