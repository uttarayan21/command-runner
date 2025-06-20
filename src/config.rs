use crate::*;
use core::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PartialConfig {
    database: Option<PathBuf>,
    host: Option<IpAddr>,
    port: Option<u16>,
}

pub struct Config {
    pub database: PathBuf,
    pub host: IpAddr,
    pub port: u16,
}

impl TryFrom<PartialConfig> for Config {
    type Error = Report<Error>;

    fn try_from(value: PartialConfig) -> Result<Self, Self::Error> {
        let database = value.database.ok_or_else(|| {
            Error::new().attach_printable("Database path is required in the configuration")
        })?;
        let host = value.host.ok_or_else(|| {
            Error::new().attach_printable("Host is required in the configuration")
        })?;
        let port = value.port.ok_or_else(|| {
            Error::new().attach_printable("Port is required in the configuration")
        })?;
        Ok(Self {
            database,
            host,
            port,
        })
    }
}

impl Config {
    pub fn try_new(cli: &cli::Cli) -> Result<Self> {
        let partial_config = PartialConfig::try_from_cli_or_env_or_file(cli)?;
        Self::try_from(partial_config).change_context(Error)
    }
}

impl Default for PartialConfig {
    fn default() -> Self {
        Self {
            database: None,
            host: Some(IpAddr::V4(core::net::Ipv4Addr::LOCALHOST)),
            port: Some(5599),
        }
    }
}

impl PartialConfig {
    pub fn from_env() -> Result<Self> {
        let database = std::env::var("CMD_RUNNER_DATABASE").ok().map(PathBuf::from);
        let host = std::env::var("CMD_RUNNER_HOST")
            .ok()
            .map(|host| host.parse::<IpAddr>())
            .transpose()
            .change_context(Error)?;
        let port = std::env::var("CMD_RUNNER_PORT")
            .ok()
            .map(|port| port.parse::<u16>())
            .transpose()
            .change_context(Error)
            .attach_printable("Failed to parse port")?;
        Ok(Self {
            database,
            host,
            port,
        })
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let Ok(content) = std::fs::read_to_string(path)
            .change_context(Error)
            .attach_printable("Failed to read config file")
            .inspect_err(|e| {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        tracing::warn!("Config file not found, using default configuration");
                    } else {
                        tracing::error!("Error reading config file: {}", e);
                    }
                } else {
                    tracing::error!("Unexpected error reading config file: {}", e);
                }
            })
        else {
            return Ok(Self::default());
        };
        let config: Self = toml::from_str(&content)
            .change_context(Error)
            .attach_printable("Failed to parse config toml file")?;
        Ok(config)
    }

    pub fn from_cli(cli: &cli::Cli) -> Result<Self> {
        let run = match &cli.cmd {
            cli::SubCommand::Run(run) => Some(run),
            _ => None,
        };
        Ok(Self {
            database: cli.database.clone(),
            host: run.and_then(|r| r.host),
            port: run.and_then(|r| r.port),
        })
    }

    pub fn or(self, other: Self) -> Self {
        Self {
            database: self.database.or(other.database),
            host: self.host.or(other.host),
            port: self.port.or(other.port),
        }
    }

    pub fn try_from_cli_or_env_or_file(cli: &cli::Cli) -> Result<Self> {
        Ok(Self::from_cli(cli)
            .change_context(Error)
            .attach_printable("Failed to read config from CLI")?
            .or(Self::from_env()
                .change_context(Error)
                .attach_printable("Failed to read config from environment variables")?)
            .or(Self::from_file(&cli.config)
                .change_context(Error)
                .attach_printable("Failed to read config from file")?)
            .or(Self::default()))
    }
}
