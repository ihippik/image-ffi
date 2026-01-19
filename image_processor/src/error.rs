use thiserror::Error;

/// Application-level errors produced by the image processor.
#[derive(Error, Debug)]
pub enum AppError {
    /// Input image file does not exist.
    #[error("Input file does not exist: {0}")]
    MissingInput(String),

    /// Params file does not exist.
    #[error("Params file does not exist: {0}")]
    MissingParams(String),

    /// Plugin dynamic library does not exist.
    #[error("Plugin library does not exist: {0}")]
    MissingPlugin(String),

    /// I/O error occurred while reading or writing files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error occurred while decoding or encoding an image.
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    /// Error occurred while loading a dynamic plugin library.
    #[error("Plugin load error: {0}")]
    Plugin(#[from] libloading::Error),

    /// Params file contains invalid UTF-8 data.
    #[error("Invalid UTF-8 in params file")]
    InvalidParamsUtf8,
}
