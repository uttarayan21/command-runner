use crate::*;
pub async fn connect(database_url: impl AsRef<str>) -> Result<sqlx::SqlitePool> {
    let path = std::path::Path::new(database_url.as_ref());
    let parent = path.parent().ok_or_else(|| {
        Error::new().attach_printable("Invalid database URL: no parent directory found").attach_printable(format!("Database URL: {}", database_url.as_ref()))
    })?;
    if !parent.exists() {
        std::fs::create_dir_all(parent).change_context(Error).attach_printable_lazy(|| format!("Failed to create database directory: {}", parent.display()))?;
    }

    let options = sqlx::sqlite::SqliteConnectOptions::default()
        .filename(path)
        .create_if_missing(true);
    let database = sqlx::SqlitePool::connect_with(options)
        .await
        .change_context(Error)?;
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .change_context(Error).attach_printable("Failed to apply sqlx migrations")?;
    Ok(database)
}
