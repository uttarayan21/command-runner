use crate::*;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Command {
    #[sqlx(try_from = "UuidWrapper")]
    pub id: Option<uuid::Uuid>,
    pub command: String,
    #[sqlx(json)]
    pub args: Vec<String>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct UuidWrapper(uuid::fmt::Hyphenated);
impl From<UuidWrapper> for Option<uuid::Uuid> {
    fn from(input: UuidWrapper) -> Self {
        Some(input.0.into_uuid())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExitStatus {
    success: bool,
    code: Option<i32>,
}

impl From<std::process::ExitStatus> for ExitStatus {
    fn from(status: std::process::ExitStatus) -> Self {
        ExitStatus {
            success: status.success(),
            code: status.code(),
        }
    }
}

impl From<std::process::Output> for Output {
    fn from(output: std::process::Output) -> Self {
        Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: ExitStatus::from(output.status),
        }
    }
}

impl Command {
    pub const fn new(command: String, args: Vec<String>) -> Self {
        Command {
            id: None,
            command,
            args,
        }
    }
    pub async fn run(&self) -> Result<Output> {
        use tokio::process::Command;
        Command::new(&self.command)
            .args(&self.args)
            .output()
            .await
            .change_context(Error)
            .attach_printable(format!(
                "Failed to run command: {} with args: {:?}",
                self.command, self.args
            ))
            .map(From::from)
    }

    pub async fn add(self, database: &sqlx::SqlitePool) -> Result<uuid::Uuid> {
        query_add(database, &self).await
    }

    pub async fn query(id: uuid::Uuid, database: &sqlx::SqlitePool) -> Result<Command> {
        query_get(database, id).await
    }
    pub async fn list(database: &sqlx::SqlitePool) -> Result<Vec<Command>> {
        query_list(database).await
    }
    pub async fn like(
        database: &sqlx::SqlitePool,
        pattern: impl AsRef<str>,
    ) -> Result<Vec<Command>> {
        query_like(database, pattern.as_ref()).await
    }
}

async fn query_get(database: &sqlx::SqlitePool, id: uuid::Uuid) -> Result<Command> {
    sqlx::query_as("SELECT id, command, args FROM commands WHERE id = ?")
        .bind(id.as_hyphenated())
        .fetch_one(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to query command with id: {}", id))
        .attach(http::StatusCode::NOT_FOUND)
}

async fn query_list(database: &sqlx::SqlitePool) -> Result<Vec<Command>> {
    sqlx::query_as("SELECT id, command, args FROM commands")
        .fetch_all(database)
        .await
        .change_context(Error)
        .attach_printable("Failed to list commands")
}

async fn query_add(database: &sqlx::SqlitePool, command: &Command) -> Result<uuid::Uuid> {
    let id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO commands (id, command, args) VALUES (?, ?, ?)")
        .bind(id.as_hyphenated())
        .bind(&command.command)
        .bind(&sqlx::types::Json(&command.args))
        .execute(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to add command: {}", command.command))
        .attach(http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(id)
}

async fn query_like(database: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Command>> {
    sqlx::query_as("SELECT id, command, args FROM commands WHERE command LIKE ?")
        .bind(format!("%{}%", pattern))
        .fetch_all(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to query commands like: {}", pattern))
}
