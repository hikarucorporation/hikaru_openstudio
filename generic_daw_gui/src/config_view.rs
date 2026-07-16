use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task, Theme};
use std::path::PathBuf;
use generic_daw::Config; // Ajustá este import según la estructura real de tus crates

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    ToggleAutosave(bool),
    ToggleStartupScan(bool),
    ChangeTheme(String), // Simplificado para el ejemplo, adaptalo a tu enum/type de Theme
    BrowseVst3Path,
    AddVst3Path(PathBuf),
    BrowseClapPath,
    AddClapPath(PathBuf),
    BrowseSamplePath,
    AddSamplePath(PathBuf),
    Save,
    Close,
}

#[derive(Debug)]
pub struct ConfigView {
    autosave_enabled: bool,
    startup_scan: bool,
    theme_name: String,
    vst3_paths: Vec<PathBuf>,
    clap_paths: Vec<PathBuf>,
    sample_paths: Vec<PathBuf>,
}

impl ConfigView {
    pub fn new(config: &Config) -> Self {
        Self {
            autosave_enabled: config.autosave_enabled,
            startup_scan: config.startup_scan,
            theme_name: config.theme_name.clone(),
            vst3_paths: config.vst3_paths.clone(),
            clap_paths: config.clap_paths.clone(),
            sample_paths: config.sample_paths.clone(),
        }
    }

    /// Reconstruye el struct Config a partir del estado actual de la UI en memoria.
    fn current_config(&self) -> Config {
        Config {
            autosave_enabled: self.autosave_enabled,
            startup_scan: self.startup_scan,
            theme_name: self.theme_name.clone(),
            vst3_paths: self.vst3_paths.clone(),
            clap_paths: self.clap_paths.clone(),
            sample_paths: self.sample_paths.clone(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Config> {
        match message {
            Message::NoOp => {
                // No hacemos nada, simplemente retornamos el estado actual sin mutar ni guardar
                Task::done(self.current_config())
            }

            Message::ToggleAutosave(enabled) => {
                self.autosave_enabled = enabled;
                Task::done(self.current_config())
            }

            Message::ToggleStartupScan(enabled) => {
                self.startup_scan = enabled;
                Task::done(self.current_config())
            }

            Message::ChangeTheme(theme) => {
                self.theme_name = theme;
                Task::done(self.current_config())
            }

            Message::BrowseVst3Path => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Seleccionar carpeta VST3")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |maybe_path| {
                        if let Some(path) = maybe_path {
                            Message::AddVst3Path(path)
                        } else {
                            Message::NoOp // Cancelado de forma segura, sin auto-guardar
                        }
                    },
                ).map(|_| self.current_config())
            }

            Message::AddVst3Path(path) => {
                if !self.vst3_paths.contains(&path) {
                    self.vst3_paths.push(path);
                }
                Task::done(self.current_config())
            }

            Message::BrowseClapPath => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Seleccionar carpeta CLAP")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |maybe_path| {
                        if let Some(path) = maybe_path {
                            Message::AddClapPath(path)
                        } else {
                            Message::NoOp
                        }
                    },
                ).map(|_| self.current_config())
            }

            Message::AddClapPath(path) => {
                if !self.clap_paths.contains(&path) {
                    self.clap_paths.push(path);
                }
                Task::done(self.current_config())
            }

            Message::BrowseSamplePath => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Seleccionar carpeta de Samples")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |maybe_path| {
                        if let Some(path) = maybe_path {
                            Message::AddSamplePath(path)
                        } else {
                            Message::NoOp
                        }
                    },
                ).map(|_| self.current_config())
            }

            Message::AddSamplePath(path) => {
                if !self.sample_paths.contains(&path) {
                    self.sample_paths.push(path);
                }
                Task::done(self.current_config())
            }

            Message::Save => {
                let config = self.current_config();
                // Persistimos en disco de forma síncrona sólo cuando se solicita explícitamente
                if let Err(e) = config.write() {
                    eprintln!("Error al guardar la configuración: {:?}", e);
                }
                Task::done(config)
            }

            Message::Close => {
                let config = self.current_config();
                if let Err(e) = config.write() {
                    eprintln!("Error al guardar la configuración antes de cerrar: {:?}", e);
                }
                // Aquí podés despachar lógica para cerrar el modal/vista además de retornar el config
                Task::done(config)
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title = text("Configuración de generic-daw")
            .size(24);

        let options = column![
            checkbox(self.autosave_enabled)
                .label("Habilitar guardado automático")
                .on_toggle(Message::ToggleAutosave), // Separado por coma, sin ';'
            checkbox(self.startup_scan)
                .label("Escanear plugins al iniciar")
                .on_toggle(Message::ToggleStartupScan), // Separado por coma
        ]
        .spacing(12);

        // Sección de rutas VST3
        let vst3_section = column![
            row![
                text("Rutas VST3").size(18),
                button("Explorar...").on_press(Message::BrowseVst3Path)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            column(
                self.vst3_paths
                    .iter()
                    .map(|p| text(format!("{}", p.display())).into())
                    .collect()
            )
            .spacing(5)
        ]
        .spacing(10);

        // Sección de rutas CLAP
        let clap_section = column![
            row![
                text("Rutas CLAP").size(18),
                button("Explorar...").on_press(Message::BrowseClapPath)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            column(
                self.clap_paths
                    .iter()
                    .map(|p| text(format!("{}", p.display())).into())
                    .collect()
            )
            .spacing(5)
        ]
        .spacing(10);

        // Sección de rutas de Samples
        let sample_section = column![
            row![
                text("Librerías de Samples").size(18),
                button("Explorar...").on_press(Message::BrowseSamplePath)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            column(
                self.sample_paths
                    .iter()
                    .map(|p| text(format!("{}", p.display())).into())
                    .collect()
            )
            .spacing(5)
        ]
        .spacing(10);

        let footer = row![
            button("Cancelar").on_press(Message::NoOp),
            button("Guardar Cambios").on_press(Message::Close),
        ]
        .spacing(20)
        .align_y(Alignment::Center);

        let content = column![
            title,
            options,
            vst3_section,
            clap_section,
            sample_section,
            footer
        ]
        .spacing(25)
        .width(600);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .padding(20)
            .into()
    }
}
