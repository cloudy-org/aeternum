use std::path::PathBuf;

use eframe::egui::{self, ImageSource};
use rfd::FileDialog;

use crate::{image::Image, Error};

pub fn get_aeternum_image<'a>() -> ImageSource<'a> {
    egui::include_image!("../assets/image.png")
}

pub fn select_image() -> Result<Image, Error> {
    let image_path = FileDialog::new()
        .add_filter("images", &["png", "jpeg", "jpg", "webp"])
        .pick_file();

    let image_or_error = match image_path {
        Some(path) => {
            if !path.exists() {
                Err(
                    Error::FileNotFound(
                        None,
                        path,
                        "The file picked in the file selector does not exist!".to_string()
                    )
                )
            } else {
                Image::from_path(path)
            }
        },
        None => Err(Error::NoFileSelected(None))
    };

    image_or_error
}

pub fn save_folder() -> Result<PathBuf, Error> {
    match FileDialog::new().pick_folder() {
        Some(path) => {
            Ok(path)
        },
        None => Err(Error::NoFileSelected(None))
    }
}