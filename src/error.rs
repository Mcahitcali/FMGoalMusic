use thiserror::Error;

/// Application-level errors using thiserror for structured error handling.
///
/// These errors represent domain-specific failures that can occur during
/// application operation. They provide context and can be chained with anyhow.

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to load audio file: {path}")]
    LoadFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to decode audio format")]
    DecodeFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to initialize audio output stream")]
    StreamInitFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Audio playback failed")]
    PlaybackFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Failed to initialize screen capturer")]
    InitFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to capture screen")]
    CaptureFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("No displays found")]
    NoDisplays,

    #[error("Invalid display index: {0}")]
    InvalidDisplayIndex(usize),
}

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Failed to initialize OCR engine")]
    InitFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to perform OCR on image")]
    RecognitionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to preprocess image")]
    PreprocessFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Text extraction failed")]
    ExtractionFailed,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration from {path}")]
    LoadFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to save configuration to {path}")]
    SaveFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to create config directory: {path}")]
    DirectoryCreationFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Error, Debug)]
pub enum TeamError {
    #[error("Failed to load team database from {path}")]
    LoadFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to save team database to {path}")]
    SaveFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Team not found: {0}")]
    NotFound(String),

    #[error("Invalid team data: {0}")]
    Invalid(String),
}

#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("Detection not running")]
    NotRunning,

    #[error("Detection already running")]
    AlreadyRunning,

    #[error("Invalid detection region: {0:?}")]
    InvalidRegion([u32; 4]),

    #[error("No music file selected")]
    NoMusicSelected,

    #[error("Failed to start detection thread")]
    ThreadSpawnFailed(#[source] std::io::Error),
}

/// Type alias for application Results using anyhow for context chaining
pub type AppResult<T> = anyhow::Result<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AudioError::InvalidFormat("unknown".to_string());
        assert_eq!(err.to_string(), "Invalid audio format: unknown");

        let err = DetectionError::NotRunning;
        assert_eq!(err.to_string(), "Detection not running");
    }

    #[test]
    fn test_error_source_chain() {
        use std::io;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let config_err = ConfigError::LoadFailed {
            path: "/test/config.json".to_string(),
            source: Box::new(io_err),
        };

        assert!(config_err.source().is_some());
        assert_eq!(
            config_err.to_string(),
            "Failed to load configuration from /test/config.json"
        );
    }
}
