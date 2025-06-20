use crate::*;
pub async fn connect(database_url: impl AsRef<str>) -> Result<sqlx::SqlitePool> {
    let options = sqlx::sqlite::SqliteConnectOptions::default()
        .filename(database_url.as_ref())
        .create_if_missing(true);
    let database = sqlx::SqlitePool::connect_with(options)
        .await
        .change_context(Error)?;
    Ok(database)
}
