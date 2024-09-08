use thiserror::Error;

pub type SbResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Scatterbrain error: {0}")]
    Scattterbrain(#[from] scatterbrain::error::Error),
    #[error("Toml serialize error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("Toml deserialize error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("Config directory missing")]
    ConfigMissingError,
    #[error("File already exist: {0}")]
    ConfigAlreadyExists(String),
    #[error("File does not exist: {0}")]
    ConfigDoesNotExist(String),
    #[error("IO error {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid argument: {0}")]
    ClapError(#[from] clap::error::Error),
    #[error("Not paired, please attempt pairing.")]
    NotPaired,
}
