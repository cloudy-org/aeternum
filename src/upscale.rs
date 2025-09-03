use std::{io::{BufRead, BufReader}, path::PathBuf, process::Stdio, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use egui_notify::ToastLevel;
use std::process::Command;
use strum_macros::{EnumIter, Display};

use crate::{app::Notifier, error::Error, image::Image};

#[derive(Clone, PartialEq, EnumIter, Display)]
pub enum OutputExt {
    #[strum(to_string = "WebP")]
    WebP,
    #[strum(to_string = "PNG")]
    PNG,
    #[strum(to_string = "JPG")]
    JPG
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    path: PathBuf,
    folder: PathBuf,

    pub name: String
}

#[derive(Clone)]
pub struct UpscaleOptions {
    pub scale: i8,
    pub compression: i32,
    pub model: Option<Model>,
    pub output_ext: OutputExt,
    pub output: Option<PathBuf>
}

pub struct Upscale {
    pub options: UpscaleOptions,
    pub upscaling: bool,
    pub models: Vec<Model>,

    models_folder: PathBuf,
    cli_path: PathBuf,
    upscaling_arc: Arc<Mutex<bool>>
}

impl Default for UpscaleOptions {
    fn default() -> Self {
        Self {
            scale: 4,
            compression: 0,
            model: None,
            output_ext: OutputExt::PNG,
            output: None
        }
    }
}

impl Upscale {
    #[cfg(feature = "package")]
    pub fn new() -> Result<Self, Error> {
        use std::env;

        let executable_path = match env::current_exe() {
            Ok(path) => path,
            Err(error) => return Err(Error::FailedToGetCurrentExecutablePath(Some(error.to_string())))
        };
        
        let tool_path = if cfg!(unix) {
            executable_path.with_file_name("upscayl-bin")
        } else {
            executable_path.with_file_name("upscayl-bin.exe")
        };

        if !tool_path.exists() {
            return Err(Error::UpscaylNotInPath(Some("upscayl-bin is not with the aeternum executable.".to_string())))
        }

        let models_folder = executable_path.with_file_name("models");

        if !models_folder.exists() {
            return Err(Error::ModelsFolderNotFound(Some("Folder does not exist.".to_string()), models_folder))
        }

        return Ok(Self {
            options: UpscaleOptions::default(),
            upscaling: false,
            models: Vec::new(),

            models_folder,
            cli_path: tool_path,
            upscaling_arc: Arc::new(false.into())
        })
    }

    #[cfg(not(feature = "package"))] // NOTE: This only works on linux.
    pub fn new() -> Result<Self, Error> {
        use which::which;

        match which("upscayl-bin") {
            Ok(path) => {
                let models_folder = PathBuf::from("/usr/lib/upscayl/models");

                if !models_folder.exists() {
                    return Err(Error::ModelsFolderNotFound(Some("Folder doesn't exist".to_string()), models_folder))
                }

                Ok(Self {
                    options: UpscaleOptions::default(),
                    upscaling: false,
                    models: Vec::new(),

                    models_folder,
                    cli_path: path,
                    upscaling_arc: Arc::new(false.into())
                })
            },
            Err(err) => Err(Error::UpscaylNotInPath(Some(err.to_string())))
        }
    }

    pub fn init(&mut self, enabled: bool) -> Result<(), Error> {
        if enabled {
            let path: PathBuf = dirs::config_local_dir().unwrap().join("cloudy").join("aeternum").join("models");

            if path.exists() {
                self.get_models(path);
            } else {
                return Err(Error::NoModels(Some("Custom folder doesn't exist.".to_string()), path))
            }
        }

        self.get_models(self.models_folder.clone());

        if self.models.is_empty() {
            return Err(Error::NoModels(Some("Vector is empty.".to_string()), self.models_folder.clone()))
        }

        Ok(())
    }

    pub fn update(&mut self) {
        if let Ok(value) = self.upscaling_arc.try_lock() {
            self.upscaling = value.clone();
        }
    }

    pub fn reset_options(&mut self) {
        self.options = UpscaleOptions::default();
    }

    fn upscaling_reset(&mut self) {
        self.upscaling = false;
        self.upscaling_arc = Arc::new(false.into());
    }

    pub fn upscale(&mut self, image: Image, notifier: &Notifier) {
        self.upscaling_reset();

        let path = &image.path;

        let output_folder = match &self.options.output {
            Some(path) => path.clone(),
            None => image.path.parent().unwrap().to_path_buf()
        };

        let out = output_folder.join(
            image.create_output(&self.options)
        );

        let path = path.clone();
        let cli_path = self.cli_path.clone();
        let upscaling_arc = self.upscaling_arc.clone();
        let notifier_arc = notifier.clone();
        let options = self.options.clone();

        let mut upscaling = self.upscaling_arc.lock().unwrap();
        *upscaling = true;

        let upscale_stuff = move || {
            let now = Instant::now();

            notifier_arc.set_loading(Some("Initializing command...".into()));

            let mut upscale_command = Command::new(cli_path.to_string_lossy().to_string());

            #[cfg(target_os = "windows")] {
                use std::os::windows::process::CommandExt;

                upscale_command.creation_flags(0x08000000);
            }

            let model = &options.model.unwrap();

            let cmd = upscale_command
                .args([
                    "-i",
                    &path.to_string_lossy(),
                    "-o",
                    &out.to_string_lossy(),
                    "-m",
                    &model.folder.to_string_lossy(),
                    "-n",
                    &model.name,
                    "-s",
                    &options.scale.to_string(),
                    "-c",
                    &options.compression.to_string()
                ])
                .stderr(Stdio::piped()) // why do you output to stderr :woe: ~ Ananas
                .spawn();

            match cmd {
                Ok(mut child) => {
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);

                        for line in reader.lines() {
                            match line {
                                Ok(output) => {
                                    let out_bytes = output.as_bytes();

                                    if !out_bytes.is_empty() && out_bytes[0].is_ascii_digit() {
                                        notifier_arc.set_loading(Some(format!("Processing: {}", output)));
                                    }
                                },
                                _ => {}
                            }
                        }
                    }

                    let status = child.wait_with_output();

                    match status {
                        Ok(status) => {
                            if status.status.success() {
                                let upscale_time = now.elapsed().as_secs();

                                notifier_arc.toast(
                                    format!("Successfully upscaled image in {} seconds!", upscale_time),
                                    ToastLevel::Success,
                                    |toast| {
                                        toast.duration(Some(Duration::from_secs(10)));
                                    }
                                );
                            } else {
                                let error = Error::FailedToUpscaleImage(
                                    None,
                                    "Process returned as not successful.".to_string()
                                );
                                notifier_arc.toast(
                                    Box::new(error),
                                    ToastLevel::Error,
                                    |toast| {
                                        toast.duration(Some(Duration::from_secs(10)));
                                    }
                                );
                            }
                        },
                        Err(error) => {
                            let error = Error::FailedToUpscaleImage(
                                Some(error.to_string()),
                                "Failed to wait for process.".to_string()
                            );

                            notifier_arc.toast(
                                Box::new(error),
                                ToastLevel::Error,
                                |toast| {
                                    toast.duration(Some(Duration::from_secs(10)));
                                }
                            );
                        }
                    }
                },
                Err(error) => {
                    let error = Error::FailedToUpscaleImage(Some(error.to_string()), "Failed to spawn child process.".to_string());

                    notifier_arc.toast(
                        Box::new(error),
                        ToastLevel::Error,
                        |toast| {
                            toast.duration(Some(Duration::from_secs(10)));
                        }
                    );
                }
            }

            notifier_arc.unset_loading();

            let mut upscaling = upscaling_arc.lock().unwrap();
            *upscaling = false;
        };

        thread::spawn(upscale_stuff);
    }

    fn get_models(&mut self, folder_path: PathBuf) {
        let glob_bin = folder_path.join("*.bin");
        let gl = glob_bin.to_string_lossy();

        for entry in glob::glob(&gl).unwrap() {
            match entry {
                Ok(entry_path) => {
                    if format!("{}", entry_path.display()).contains("video") {
                        continue;
                    }

                    let param_file = entry_path.with_file_name(
                        format!("{}.param", entry_path.file_stem().unwrap().to_string_lossy())
                    );

                    if param_file.exists() {
                        self.models.push(
                            Model {
                                path: entry_path.clone(),
                                folder: folder_path.clone(),

                                name: entry_path.file_stem().unwrap().to_string_lossy().to_string()
                            }
                        );
                    }
                },
                Err(err) => {
                    panic!("Error while getting models: {}", err);
                }
            }
        }
    }
}
