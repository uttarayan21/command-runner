use crate::*;
use regex::Regex;

use std::{collections::BTreeMap, sync::LazyLock};
static REPLACE_WITH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{.*\}").expect("Failed to compile regex"));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Command {
    #[sqlx(try_from = "UuidWrapper")]
    pub id: uuid::Uuid,
    pub name: String,
    pub command: String,
    #[sqlx(json)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Identifier {
    Id(uuid::Uuid),
    Name(String),
    Like(String),
}

#[derive(Debug, Clone)]
pub struct IdentifierWrapper(Identifier);
impl From<IdentifierWrapper> for Identifier {
    fn from(wrapper: IdentifierWrapper) -> Self {
        wrapper.0
    }
}
impl From<Identifier> for IdentifierWrapper {
    fn from(identifier: Identifier) -> Self {
        IdentifierWrapper(identifier)
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        pub struct IdentifierStruct {
            pub like: Option<String>,
            pub id: Option<uuid::Uuid>,
            pub name: Option<String>,
        }
        let identifier = IdentifierStruct::deserialize(deserializer)?;
        if let Some(id) = identifier.id {
            Ok(Identifier::Id(id))
        } else if let Some(name) = identifier.name {
            Ok(Identifier::Name(name))
        } else if let Some(like) = identifier.like {
            Ok(Identifier::Like(like))
        } else {
            Err(serde::de::Error::custom("No valid identifier provided"))
        }
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct UuidWrapper(uuid::fmt::Simple);
impl From<UuidWrapper> for uuid::Uuid {
    fn from(input: UuidWrapper) -> Self {
        input.0.into_uuid()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
pub enum CommandAddMode {
    Ignore,
    Replace,
    #[default]
    Error,
}

impl Output {
    pub async fn save(&self, database: &sqlx::SqlitePool, command_id: uuid::Uuid) -> Result<()> {
        sqlx::query(
            "INSERT INTO command_outputs (command_id, stdout, stderr, success, code) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(command_id)
        .bind(&self.stdout)
        .bind(&self.stderr)
        .bind(self.status.success)
        .bind(self.status.code)
        .execute(database)
        .await
        .change_context(Error)
        .attach_printable(format!(
            "Failed to save output for command id: {}",
            command_id
        ))?;
        Ok(())
    }
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
        use ::tap::*;
        Output {
            stdout: String::from_utf8_lossy(&output.stdout)
                .tap(|f| {
                    tracing::debug!("Command stdout: {}", f);
                })
                .to_string(),
            stderr: String::from_utf8_lossy(&output.stderr)
                .tap(|f| {
                    tracing::debug!("Command stdout: {}", f);
                })
                .to_string(),
            status: ExitStatus::from(output.status),
        }
    }
}

impl Command {
    pub const fn new(name: String, command: String, args: Vec<String>) -> Self {
        Command {
            id: uuid::Uuid::nil(),
            name,
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
                "Failed to run command: {} with args: {}",
                self.command,
                self.args.join(" ")
            ))
            .map(From::from)
    }

    pub async fn run_with_placeholder(&self, args: BTreeMap<String, String>) -> Result<Output> {
        if args.is_empty() {
            return self.run().await;
        }

        let args = self
            .args
            .iter()
            .map(|arg| {
                if REPLACE_WITH.is_match(arg) {
                    args.get(arg)
                        .ok_or_else(|| {
                            Error::new().attach_printable(format!(
                                "Not enough arguments provided for command: {}",
                                self.command
                            ))
                        })
                        .and_then(|value| {
                            let replaced_arg = REPLACE_WITH.replace_all(arg, value).to_string();
                            if replaced_arg.is_empty() {
                                Err(Error::new().attach_printable(format!(
                                    "Replacement resulted in an empty argument for command: {}",
                                    self.command
                                )))
                            } else {
                                Ok(replaced_arg)
                            }
                        })
                } else {
                    Ok(arg.to_string())
                }
            })
            .collect::<Result<Vec<_>>>()?;

        use tokio::process::Command;
        Command::new(&self.command)
            .args(&args)
            .output()
            .await
            .change_context(Error)
            .attach_printable(format!(
                "Failed to run command: {} with args: {}",
                self.command,
                args.join(" ")
            ))
            .map(From::from)
    }

    pub async fn add(
        self,
        database: &sqlx::SqlitePool,
        mode: CommandAddMode,
    ) -> Result<uuid::Uuid> {
        query_add(database, &self, mode).await
    }

    // pub async fn query(id: uuid::Uuid, database: &sqlx::SqlitePool) -> Result<Command> {
    //     query_get(database, id).await
    // }
    pub async fn delete(&self, database: &sqlx::SqlitePool) -> Result<()> {
        query_delete(database, self.id).await
    }

    pub async fn delete_all(database: &sqlx::SqlitePool) -> Result<()> {
        delete_all(database).await
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
    pub async fn identifier(
        database: &sqlx::SqlitePool,
        identifier: Identifier,
    ) -> Result<Command> {
        query_identifier(database, identifier).await
    }
}

async fn query_get(database: &sqlx::SqlitePool, id: uuid::Uuid) -> Result<Command> {
    sqlx::query_as("SELECT id,name, command, args FROM commands WHERE id = ?")
        .bind(id)
        .fetch_one(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to query command with id: {}", id))
        .attach(http::StatusCode::NOT_FOUND)
}

async fn query_list(database: &sqlx::SqlitePool) -> Result<Vec<Command>> {
    sqlx::query_as("SELECT id,name, command, args FROM commands")
        .fetch_all(database)
        .await
        .change_context(Error)
        .attach_printable("Failed to list commands")
}

async fn query_add(
    database: &sqlx::SqlitePool,
    command: &Command,
    mode: CommandAddMode,
) -> Result<uuid::Uuid> {
    let id = uuid::Uuid::new_v4();
    match mode {
        CommandAddMode::Ignore => sqlx::query(
            "INSERT OR IGNORE INTO commands (id, name, command, args) VALUES (?, ?, ?, ?)",
        ),
        CommandAddMode::Replace => sqlx::query(
            "INSERT OR REPLACE INTO commands (id, name, command, args) VALUES (?, ?, ?, ?)",
        ),
        CommandAddMode::Error => {
            sqlx::query("INSERT INTO commands (id, name, command, args) VALUES (?, ?, ?, ?)")
        }
    }
    .bind(id.as_simple())
    .bind(&command.name)
    .bind(&command.command)
    .bind(sqlx::types::Json(&command.args))
    .execute(database)
    .await
    .change_context(Error)
    .attach_printable(format!("Failed to add command: {}", command.command))
    .attach(http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(id)
}

async fn query_like(database: &sqlx::SqlitePool, pattern: &str) -> Result<Vec<Command>> {
    let pattern_bind = format!("%{}%", pattern);
    let out = sqlx::query_as(
        "SELECT id,name, command, args FROM commands WHERE command LIKE ? OR name LIKE ?",
    )
    .bind(&pattern_bind)
    .bind(&pattern_bind)
    .fetch_all(database)
    .await
    .change_context(Error)
    .attach_printable(format!("Failed to query commands like: {}", pattern))?;
    if out.is_empty() {
        return Err(Error)
            .attach_printable(format!("No commands found matching pattern: {}", pattern))
            .attach(http::StatusCode::NOT_FOUND);
    }
    Ok(out)
}

async fn query_name(database: &sqlx::SqlitePool, name: &str) -> Result<Command> {
    sqlx::query_as("SELECT id,name, command, args FROM commands WHERE name = ?")
        .bind(name)
        .fetch_one(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to query command with name: {}", name))
        .attach(http::StatusCode::NOT_FOUND)
}

async fn query_delete(database: &sqlx::SqlitePool, id: uuid::Uuid) -> Result<()> {
    sqlx::query("DELETE FROM commands WHERE id = ?")
        .bind(id)
        .execute(database)
        .await
        .change_context(Error)
        .attach_printable(format!("Failed to delete command with id: {}", id))?;
    Ok(())
}

async fn delete_all(database: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM commands")
        .execute(database)
        .await
        .change_context(Error)
        .attach_printable("Failed to delete all commands")?;
    Ok(())
}

async fn query_identifier(database: &sqlx::SqlitePool, identifier: Identifier) -> Result<Command> {
    match identifier {
        Identifier::Id(uuid) => query_get(database, uuid).await,
        Identifier::Name(name) => query_name(database, &name).await,
        Identifier::Like(pattern) => Ok(query_like(database, &pattern).await?[0].clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let identifier = Identifier::Id(uuid);
        let id = serde_urlencoded::to_string(&identifier).expect("Failed to serialize Identifier");
        assert_eq!(id, format!(r#"type=Id&value={}"#, uuid));
    }
}
