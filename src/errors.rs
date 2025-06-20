use axum::response::IntoResponse;
pub use error_stack::{Report, ResultExt};
#[derive(Debug, thiserror::Error)]
#[error("An error occurred")]
pub struct Error;

pub type Result<T, E = error_stack::Report<Error>> = core::result::Result<T, E>;

#[derive(Debug, serde::Serialize)]
#[serde(transparent)]
pub struct ErrorResponse(Report<Error>);

impl From<Report<Error>> for ErrorResponse {
    fn from(report: Report<Error>) -> Self {
        ErrorResponse(report)
    }
}

impl From<Error> for ErrorResponse {
    fn from(error: Error) -> Self {
        ErrorResponse(Report::new(error))
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = self
            .0
            .downcast_ref::<http::StatusCode>()
            .cloned()
            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
        // let status = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        let response_json =
            serde_json::to_string_pretty(&self).expect("Failed to serialize error response");
        (status, response_json).into_response()
    }
}
