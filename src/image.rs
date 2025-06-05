use std::path::PathBuf;
use imagesize::ImageSize;

use crate::{upscale::UpscaleOptions, Error};

#[derive(Clone)]
pub struct Image {
    pub path: PathBuf,
    pub image_size: ImageSize
}

impl Image {
    pub fn from_path(path: PathBuf) -> Result<Self, Error> {
        if let Some(extension) = path.extension() {
            let allowed_extensions = vec!["png", "jpeg", "jpg", "webp"];
            let extension_string = extension.to_string_lossy().to_string().to_lowercase();

            match allowed_extensions.iter().any(|e| extension_string.contains(e)) {
                true => {
                    let image_size = match imagesize::size(&path) {
                        Ok(size) => size,
                        Err(why) => return Err(
                            Error::FailedToInitImage(
                                Some(why.to_string()), 
                                path.clone(), 
                                "Failed to get image size.".to_string()
                            )
                        )
                    };

                    Ok(Self {
                        path,
                        image_size
                    })
                },
                false => Err(Error::ImageFormatNotSupported(None, extension_string.clone())),
            }
        } else {
            Err(Error::ImageFormatNotSupported(None, "".to_string()))
        }
    }

    pub fn create_output(&self, options: &UpscaleOptions) -> String {
        let model = &options.model.clone().unwrap();
        let extension = &options.output_ext.to_string().to_lowercase();

        format!(
            "{}_{}_x{}.{}", 
            self.path.file_stem().unwrap().to_string_lossy(), 
            model.name, 
            &options.scale,
            extension
        )
    }
}