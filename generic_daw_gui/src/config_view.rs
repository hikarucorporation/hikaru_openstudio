use crate::config::Config;
use iced::{
    Element, Task, Length,
    widget::{column, row, text, button, container, scrollable, space::Space},
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    AddVst3Path(PathBuf),
    RemoveVst3Path(usize),
    AddClapPath(PathBuf),
    RemoveClapPath(usize),
    Vst3Drag(sweeten::widget::drag::DragEvent),
    ClapDrag(sweeten::widget::drag::DragEvent),
    Save,
}

#[derive(Debug)]
pub struct ConfigView {
    vst3_paths: Vec<Arc<Path>>,
    clap_paths: Vec<Arc<Path>>,
    window_id: iced::window::Id,
}

impl ConfigView {
    pub fn new(window_id: iced::window::Id) -> Self {
        let config = Config::read();
        Self {
            vst3_paths: config.vst3_paths.clone(),
            clap_paths: config.clap_paths.clone(),
            window_id,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Config> {
        match message {
            Message::AddVst3Path(path) => {
                self.vst3_paths.push(Arc::from(path));
            }
            Message::RemoveVst3Path(index) => {
                self.vst3_paths.remove(index);
            }
            Message::AddClapPath(path) => {
                self.clap_paths.push(Arc::from(path));
            }
            Message::RemoveClapPath(index) => {
                self.clap_paths.remove(index);
            }
            Message::Vst3Drag(_drag_event) => {
                // Temporalmente ignorado para evitar errores de firmas en sweeten
            }
            Message::ClapDrag(_drag_event) => {
                // Temporalmente ignorado para evitar errores de firmas en sweeten
            }
            Message::Save => {
                let mut config = Config::read();
                config.vst3_paths = self.vst3_paths.clone();
                config.clap_paths = self.clap_paths.clone();
                config.write();
                return Task::done(config);
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            text("Configuración de Rutas").size(24),
            Space::new().height(20.0),
            text("Rutas VST3:"),
            row(self.vst3_paths.iter().enumerate().map(|(i, path)| {
                row![
                    text(path.to_string_lossy().into_owned()),
                    button("Remover").on_press(Message::RemoveVst3Path(i))
                ].spacing(10).into()
            })).spacing(10),
            Space::new().height(20.0),
            text("Rutas CLAP:"),
            row(self.clap_paths.iter().enumerate().map(|(i, path)| {
                row![
                    text(path.to_string_lossy().into_owned()),
                    button("Remover").on_press(Message::RemoveClapPath(i))
                ].spacing(10).into()
            })).spacing(10),
            Space::new().height(20.0),
            button("Guardar Configuración").on_press(Message::Save)
        ].spacing(10);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn keybinds(
        key: &iced::keyboard::Key,
        _modifiers: iced::keyboard::Modifiers,
        _repeat: bool,
    ) -> Option<Message> {
        match key.as_ref() {
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Save),
            _ => None,
        }
    }
}
