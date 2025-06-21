use std::collections::BTreeMap;

use crate::{command::Command, *};
use axum::Extension;

pub fn routes() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(root))
        .nest("/commands", commands())
}
type Result<T> = std::result::Result<T, ErrorResponse>;

pub async fn root() -> &'static str {
    "Command runner API"
}

pub fn commands() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(list_commands))
        .route("/search", axum::routing::get(identifier_command))
        .route("/run", axum::routing::post(run_identifier_command))
        .route("/", axum::routing::delete(delete_identifier_command))
}
pub async fn handler_404(uri: http::Uri) -> Result<()> {
    Err(Error)
        .change_context(Error)
        .attach_printable_lazy(|| format!("The specified route: {uri} doesn't exist"))
        .attach(http::StatusCode::NOT_FOUND)?;
    Ok(())
}

#[cfg(debug_assertions)]
pub async fn handler_405(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let uri = request.uri().to_owned();
    let method = request.method().to_owned();
    let response = next.run(request).await;
    if response.status() == http::StatusCode::METHOD_NOT_ALLOWED {
        use axum::response::IntoResponse;
        ErrorResponse::from(
            Report::new(Error)
                .attach_printable({
                    format!("The specified route: {uri} doesn't use the {method} method")
                })
                .attach(http::StatusCode::METHOD_NOT_ALLOWED),
        )
        .into_response()
    } else {
        response
    }
}

pub async fn list_commands(
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<axum::Json<Vec<Command>>> {
    Ok(axum::Json(Command::list(&db).await?))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RunArgs {
    history: bool,
    full: bool,
    json: bool,
}

impl Default for RunArgs {
    fn default() -> Self {
        RunArgs {
            history: true,
            full: false,
            json: false,
        }
    }
}

pub async fn identifier_command(
    axum::extract::Query(identifier): axum::extract::Query<command::Identifier>,
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<axum::Json<Command>> {
    Ok(axum::Json(Command::identifier(&db, identifier).await?))
}
pub async fn run_identifier_command(
    axum::extract::Query(identifier): axum::extract::Query<command::Identifier>,
    axum::extract::Query(run_args): axum::extract::Query<RunArgs>,
    Extension(db): Extension<sqlx::SqlitePool>,
    axum::extract::Json(args): axum::extract::Json<BTreeMap<String, String>>,
) -> Result<axum::response::Response> {
    use axum::response::IntoResponse;
    let command = Command::identifier(&db, identifier).await?;
    let output = command.run_with_placeholder(args).await?;
    if run_args.history {
        output.save(&db, command.id).await?;
    }
    if run_args.full {
        Ok(axum::Json(output).into_response())
    } else if run_args.json {
        Ok(axum::Json(
            serde_json::value::RawValue::from_string(output.stdout)
                .change_context(Error)
                .attach_printable("Failed to parse output.stdout as json")?,
        )
        .into_response())
    } else {
        Ok(output.stdout.into_response())
    }
}

pub async fn delete_identifier_command(
    axum::extract::Query(id): axum::extract::Query<command::Identifier>,
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<()> {
    let command = Command::identifier(&db, id).await?;
    command.delete(&db).await.change_context(Error)?;
    Ok(())
}
