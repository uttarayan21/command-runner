use crate::{
    command::{Command, Output},
    *,
};
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
        .route("/{id}", axum::routing::get(get_command))
        .route("/{id}/run", axum::routing::post(run_command))
        .route("/like", axum::routing::get(like_commands))
}
pub async fn handler_404(uri: http::Uri) -> Result<()> {
    Err(Error)
        .change_context(Error)
        .attach_printable_lazy(|| format!("The specified route: {uri} doesn't exist"))
        .attach(http::StatusCode::NOT_FOUND)?;
    Ok(())
}

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

pub async fn get_command(
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<axum::Json<Command>> {
    Ok(axum::Json(Command::query(id, &db).await?))
}

pub async fn run_command(
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<axum::Json<Output>> {
    let command = Command::query(id, &db).await?;
    let output = command.run().await?;
    Ok(axum::Json(output))
}

#[derive(Debug, serde::Deserialize)]
pub struct LikeCommand {
    pub pattern: String,
}

pub async fn like_commands(
    axum::extract::Query(pattern): axum::extract::Query<LikeCommand>,
    Extension(db): Extension<sqlx::SqlitePool>,
) -> Result<axum::Json<Vec<Command>>> {
    Ok(axum::Json(Command::like(&db, pattern.pattern).await?))
}
