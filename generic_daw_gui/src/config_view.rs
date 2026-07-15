use crate::config::Config;
use crate::theme::Theme;
use iced::{
    Element, Task, Length, Color, Alignment,
    widget::{
        column, row, text, button, container, scrollable, 
        checkbox, text_input, space::Space
    },
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    // Inputs de texto manuales
    Vst3InputChanged(String),
    ClapInputChanged(String),
    SampleInputChanged(String),

    // Agregar rutas escritas manualmente
    AddManualVst3,
    AddManualClap,
    AddManualSample,

    // Diálogos de Dolphin (Asíncronos)
    BrowseVst3Path,
    BrowseClapPath,
    BrowseSamplePath,

    // Acciones de las listas
    AddVst3Path(PathBuf),
    RemoveVst3Path(usize),
    AddClapPath(PathBuf),
    RemoveClapPath(usize),
    AddSamplePath(PathBuf),
    RemoveSamplePath(usize),

    // Ajustes generales
    ThemeSelected(Theme),
    ToggleAutosave(bool),
    ToggleOpenLastProject(bool),
    
    // Guardar y Cerrar
    Save,
    Close, // Mensaje para que la vista principal sepa que tiene que cerrar el modal
}

#[derive(Debug)]
pub struct ConfigView {
    vst3_paths: Vec<Arc<Path>>,
    clap_paths: Vec<Arc<Path>>,
    sample_paths: Vec<Arc<Path>>,
    theme: Theme,
    autosave_enabled: bool,
    open_last_project: bool,
    window_id: iced::window::Id,

    // Buffers para escribir en las cajas de texto
    vst3_input_buffer: String,
    clap_input_buffer: String,
    sample_input_buffer: String,
}

impl ConfigView {
    pub fn new(window_id: iced::window::Id) -> Self {
        let config = Config::read();
        Self {
            vst3_paths: config.vst3_paths.clone(),
            clap_paths: config.clap_paths.clone(),
            sample_paths: config.sample_paths.clone(),
            theme: config.theme,
            autosave_enabled: config.autosave.enabled,
            open_last_project: config.open_last_project,
            window_id,
            vst3_input_buffer: String::new(),
            clap_input_buffer: String::new(),
            sample_input_buffer: String::new(),
        }
    }

    fn current_config(&self) -> Config {
        let mut config = Config::read();
        config.vst3_paths = self.vst3_paths.clone();
        config.clap_paths = self.clap_paths.clone();
        config.sample_paths = self.sample_paths.clone();
        config.theme = self.theme;
        config.autosave.enabled = self.autosave_enabled;
        config.open_last_project = self.open_last_project;
        config
    }

    pub fn update(&mut self, message: Message) -> Task<Config> {
        match message {
            // Manejo de escritura manual en los inputs
            Message::Vst3InputChanged(val) => {
                self.vst3_input_buffer = val;
                return Task::done(self.current_config());
            }
            Message::ClapInputChanged(val) => {
                self.clap_input_buffer = val;
                return Task::done(self.current_config());
            }
            Message::SampleInputChanged(val) => {
                self.sample_input_buffer = val;
                return Task::done(self.current_config());
            }

            // Agregar lo que se escribió a mano
            Message::AddManualVst3 => {
                if !self.vst3_input_buffer.trim().is_empty() {
                    let path = PathBuf::from(self.vst3_input_buffer.trim());
                    self.vst3_paths.push(Arc::from(path));
                    self.vst3_input_buffer.clear();
                    let config = self.current_config();
                    config.write();
                    return Task::done(config);
                }
                return Task::done(self.current_config());
            }
            Message::AddManualClap => {
                if !self.clap_input_buffer.trim().is_empty() {
                    let path = PathBuf::from(self.clap_input_buffer.trim());
                    self.clap_paths.push(Arc::from(path));
                    self.clap_input_buffer.clear();
                    let config = self.current_config();
                    config.write();
                    return Task::done(config);
                }
                return Task::done(self.current_config());
            }
            Message::AddManualSample => {
                if !self.sample_input_buffer.trim().is_empty() {
                    let path = PathBuf::from(self.sample_input_buffer.trim());
                    self.sample_paths.push(Arc::from(path));
                    self.sample_input_buffer.clear();
                    let config = self.current_config();
                    config.write();
                    return Task::done(config);
                }
                return Task::done(self.current_config());
            }

            // Diálogos asíncronos integrados
            Message::BrowseVst3Path => {
                return Task::perform(
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
                            Message::Save
                        }
                    }
                ).map(|msg| {
                    // El truco definitivo: interceptamos el mensaje de callback aquí y devolvemos la Config
                    let config = Config::read();
                    config
                });
            }
            Message::AddVst3Path(path) => {
                self.vst3_paths.push(Arc::from(path));
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::RemoveVst3Path(index) => {
                self.vst3_paths.remove(index);
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }

            Message::BrowseClapPath => {
                return Task::perform(
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
                            Message::Save
                        }
                    }
                ).map(|_| {
                    let config = Config::read();
                    config
                });
            }
            Message::AddClapPath(path) => {
                self.clap_paths.push(Arc::from(path));
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::RemoveClapPath(index) => {
                self.clap_paths.remove(index);
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }

            Message::BrowseSamplePath => {
                return Task::perform(
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
                            Message::Save
                        }
                    }
                ).map(|_| {
                    let config = Config::read();
                    config
                });
            }
            Message::AddSamplePath(path) => {
                self.sample_paths.push(Arc::from(path));
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::RemoveSamplePath(index) => {
                self.sample_paths.remove(index);
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }

            Message::ThemeSelected(theme) => {
                self.theme = theme;
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::ToggleAutosave(enabled) => {
                self.autosave_enabled = enabled;
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::ToggleOpenLastProject(open) => {
                self.open_last_project = open;
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }

            Message::Save => {
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
            Message::Close => {
                let config = self.current_config();
                config.write();
                return Task::done(config);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // --- SECCIÓN: APARIENCIA ---
        let theme_section = column![
            text("Apariencia").size(16),
            Space::new().height(5.0),
            row![
                text("Tema activo:").size(13),
                Space::new().width(15.0),
                button(text(format!("{:?}", self.theme)).size(12))
                    .padding([6, 12])
                    .on_press(Message::ThemeSelected(Theme::CatppuccinFrappe))
            ].align_y(Alignment::Center)
        ];

        // --- SECCIÓN: OPCIONES GENERALES ---
        let general_section = column![
            text("Opciones Generales").size(16),
            Space::new().height(8.0),
            checkbox(self.autosave_enabled)
                .label("Habilitar Auto-guardado")
                .on_toggle(Message::ToggleAutosave)
                .size(16),
            Space::new().height(8.0),
            checkbox(self.open_last_project)
                .label("Abrir último proyecto al iniciar")
                .on_toggle(Message::ToggleOpenLastProject)
                .size(16),
        ];

        // --- SECCIÓN: RUTAS DE BÚSQUEDA ---
        let mut paths_section = column![
            text("Rutas de Plugins & Recursos").size(16),
            Space::new().height(15.0),
        ].spacing(15);

        // --- VST3 ---
        let mut vst3_column = column![
            row![
                text("Rutas VST3").size(14),
                Space::new().width(Length::Fill),
                button("Explorar con Dolphin")
                    .on_press(Message::BrowseVst3Path)
                    .padding([4, 8])
            ].align_y(Alignment::Center).width(Length::Fill)
        ].spacing(5);

        vst3_column = vst3_column.push(
            row![
                text_input(
                    "Escribí o pegá una ruta VST3 manual aquí...",
                    &self.vst3_input_buffer
                )
                .on_input(Message::Vst3InputChanged)
                .on_submit(Message::AddManualVst3)
                .padding(8)
                .size(12),
                Space::new().width(8),
                button("Agregar")
                    .on_press(Message::AddManualVst3)
                    .padding([6, 12])
            ].align_y(Alignment::Center)
        );

        for (i, path) in self.vst3_paths.iter().enumerate() {
            vst3_column = vst3_column.push(
                container(
                    row![
                        text(path.to_string_lossy().into_owned())
                            .size(11)
                            .font(iced::Font::MONOSPACE),
                        Space::new().width(Length::Fill),
                        button("Remover")
                            .on_press(Message::RemoveVst3Path(i))
                            .padding([2, 6])
                            .style(button::danger)
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                )
                .padding(6)
                .style(|theme| container::bordered_box(theme))
                .width(Length::Fill)
            );
        }
        paths_section = paths_section.push(vst3_column);

        // --- CLAP ---
        let mut clap_column = column![
            row![
                text("Rutas CLAP").size(14),
                Space::new().width(Length::Fill),
                button("Explorar con Dolphin")
                    .on_press(Message::BrowseClapPath)
                    .padding([4, 8])
            ].align_y(Alignment::Center).width(Length::Fill)
        ].spacing(5);

        clap_column = clap_column.push(
            row![
                text_input(
                    "Escribí o pegá una ruta CLAP manual aquí...",
                    &self.clap_input_buffer
                )
                .on_input(Message::ClapInputChanged)
                .on_submit(Message::AddManualClap)
                .padding(8)
                .size(12),
                Space::new().width(8),
                button("Agregar")
                    .on_press(Message::AddManualClap)
                    .padding([6, 12])
            ].align_y(Alignment::Center)
        );

        for (i, path) in self.clap_paths.iter().enumerate() {
            clap_column = clap_column.push(
                container(
                    row![
                        text(path.to_string_lossy().into_owned())
                            .size(11)
                            .font(iced::Font::MONOSPACE),
                        Space::new().width(Length::Fill),
                        button("Remover")
                            .on_press(Message::RemoveClapPath(i))
                            .padding([2, 6])
                            .style(button::danger)
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                )
                .padding(6)
                .style(|theme| container::bordered_box(theme))
                .width(Length::Fill)
            );
        }
        paths_section = paths_section.push(clap_column);

        // --- SAMPLES ---
        let mut samples_column = column![
            row![
                text("Rutas de Samples").size(14),
                Space::new().width(Length::Fill),
                button("Explorar con Dolphin")
                    .on_press(Message::BrowseSamplePath)
                    .padding([4, 8])
            ].align_y(Alignment::Center).width(Length::Fill)
        ].spacing(5);

        samples_column = samples_column.push(
            row![
                text_input(
                    "Escribí o pegá una ruta de Samples manual aquí...",
                    &self.sample_input_buffer
                )
                .on_input(Message::SampleInputChanged)
                .on_submit(Message::AddManualSample)
                .padding(8)
                .size(12),
                Space::new().width(8),
                button("Agregar")
                    .on_press(Message::AddManualSample)
                    .padding([6, 12])
            ].align_y(Alignment::Center)
        );

        for (i, path) in self.sample_paths.iter().enumerate() {
            samples_column = samples_column.push(
                container(
                    row![
                        text(path.to_string_lossy().into_owned())
                            .size(11)
                            .font(iced::Font::MONOSPACE),
                        Space::new().width(Length::Fill),
                        button("Remover")
                            .on_press(Message::RemoveSamplePath(i))
                            .padding([2, 6])
                            .style(button::danger)
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                )
                .padding(6)
                .style(|theme| container::bordered_box(theme))
                .width(Length::Fill)
            );
        }
        paths_section = paths_section.push(samples_column);

        // --- CONTENEDOR INTERNO DE CONFIGURACIÓN ---
        let content = column![
            text("Configuración Global").size(22),
            Space::new().height(15.0),
            theme_section,
            Space::new().height(15.0),
            general_section,
            Space::new().height(15.0),
            paths_section,
            Space::new().height(20.0),
            row![
                Space::new().width(Length::Fill),
                button("Guardar Cambios")
                    .on_press(Message::Close) // Dispara el cierre directamente al presionar
                    .padding([10, 20])
                    .style(button::success)
            ].width(Length::Fill)
        ].width(550);

        // --- CONTENEDOR MODAL FLOTANTE CENTRADO ---
        container(
            container(scrollable(content))
                .padding(25)
                .style(|theme| {
                    let mut base_style = container::bordered_box(theme);
                    base_style.background = Some(iced::Background::Color(Color::from_rgb8(24, 24, 27)));
                    base_style.border.radius = 10.0.into();
                    base_style
                })
                .width(600)
                .height(480)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| {
            let mut base_style = container::Style::default();
            base_style.background = Some(iced::Background::Color(Color::from_rgba8(0, 0, 0, 0.6)));
            base_style
        })
        .into()
    }

    pub fn keybinds(
        key: &iced::keyboard::Key,
        _modifiers: iced::keyboard::Modifiers,
        _repeat: bool,
    ) -> Option<Message> {
        match key.as_ref() {
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Close),
            _ => None,
        }
    }
}
