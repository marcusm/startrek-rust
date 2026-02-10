use crate::models::constants::Device;
use std::fmt;

/// Game-specific error types
#[derive(Debug)]
pub enum GameError {
    /// Failed to parse user input
    ParseError(String),
    /// Invalid input provided by user
    InvalidInput(String),
    /// Attempted to use a damaged device
    DeviceDamaged(Device),
    /// Insufficient resources (energy, torpedoes, etc.)
    InsufficientResources { required: f64, available: f64 },
    /// Navigation-related error
    NavigationError(String),
    /// I/O error occurred
    IoError(std::io::Error),
}

/// Type alias for Results using GameError
pub type GameResult<T> = Result<T, GameError>;

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GameError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            GameError::DeviceDamaged(device) => {
                write!(f, "{} is damaged and cannot be used", device.name())
            }
            GameError::InsufficientResources { required, available } => {
                write!(
                    f,
                    "Insufficient resources: required {}, available {}",
                    required, available
                )
            }
            GameError::NavigationError(msg) => write!(f, "Navigation error: {}", msg),
            GameError::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for GameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GameError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GameError {
    fn from(err: std::io::Error) -> Self {
        GameError::IoError(err)
    }
}

impl From<std::num::ParseFloatError> for GameError {
    fn from(err: std::num::ParseFloatError) -> Self {
        GameError::ParseError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for GameError {
    fn from(err: std::num::ParseIntError) -> Self {
        GameError::ParseError(err.to_string())
    }
}
