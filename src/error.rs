use std::{fmt::{self, Display, Formatter}, path::PathBuf};

use cirrus_error::v1::error::CError;
use cirrus_egui::v1::error::EguiCError;

type AE = Option<String>;

#[derive(Debug, Clone)]
pub enum Error {
    FileNotFound(AE, PathBuf, String),
    NoFileSelected(AE),
    FailedToUpscaleImage(AE, String),
    UpscaylNotInPath(AE),
    ModelsFolderNotFound(AE, PathBuf),
    NoModels(AE, PathBuf),
    FailedToInitImage(AE, PathBuf, String),
    ImageFormatNotSupported(AE, String),
    FailedToGetCurrentExecutablePath(AE)
}

impl EguiCError for Error {}

impl CError for Error {
    fn human_message(&self) -> String {
        format!("{}", self)
    }

    fn actual_error(&self) -> Option<String> {
        match self.to_owned() {
            Error::FileNotFound(actual_error, _, _) => actual_error,
            Error::NoFileSelected(actual_error) => actual_error,
            Error::FailedToUpscaleImage(actual_error, _) => actual_error,
            Error::UpscaylNotInPath(actual_error) => actual_error,
            Error::ModelsFolderNotFound(actual_error, _) => actual_error,
            Error::NoModels(actual_error, _) => actual_error,
            Error::FailedToInitImage(actual_error, _, _) => actual_error,
            Error::ImageFormatNotSupported(actual_error, _) => actual_error,
            Error::FailedToGetCurrentExecutablePath(actual_error) => actual_error,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::FileNotFound(_, path, detail) => {
                let message = format!(
                    "The file path given '{}' does not exist! {}",
                    path.to_string_lossy(),
                    detail
                );

                write!(f, "{}", message)
            },
            Error::NoFileSelected(_) => write!(
                f, "No file was selected in the file dialogue!"
            ),
            Error::FailedToUpscaleImage(_, reason) => write!(
                f,
                "Failed to upscale the image. \
                \n\nReason: {}",
                reason
            ),
            Error::FailedToInitImage(_, path, reason) => write!(
                f,
                "Failed to initialize the image ({})! Reason: {}",
                path.file_name().unwrap().to_string_lossy(),
                reason
            ),
            Error::UpscaylNotInPath(..) => write!(
                f, "upscayl-bin isn't in your path. Install it: https://github.com/upscayl/upscayl-ncnn"
            ),
            Error::ModelsFolderNotFound(_, path) => write!(
                f, "Models folder not found: {}", path.display()
            ),
            Error::NoModels(_, path) => write!(
                f, "No models found in folder: '{}'", path.display()
            ),
            Error::ImageFormatNotSupported(_, image_format) => write!(
                f, "The image format '{}' is not supported!", image_format
            ),
            Error::FailedToGetCurrentExecutablePath(_) => write!(
                f, "Failed to get the current path where aeternum is located."
            ),
        }
    }
}
