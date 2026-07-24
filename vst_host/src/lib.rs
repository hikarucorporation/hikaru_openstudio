// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod component_handler;
pub mod audio_thread;
pub mod error;
pub mod factory;
pub mod host_application;
pub mod module;
pub mod plugin;
pub mod plugin_descriptor;
pub mod scan;
pub mod shared;

pub use plugin_descriptor::PluginDescriptor;
pub use scan::get_installed_plugins;

/// Mensajes enviados desde el hilo de audio (o desde callbacks disparados por
/// el plugin) hacia el hilo principal, análogo a `clap_host::MainThreadMessage`.
///
/// Todavía minimalista: hoy nada construye/envía estos mensajes porque
/// `IComponentHandler` (quien los dispararía vía `restart_component`) no está
/// implementado aún. Se agranda variante por variante a medida que se cablea
/// cada callback real, en vez de precompletar variantes sin consumidor.
#[derive(Clone, Copy, Debug)]
pub enum MainThreadMessage {
    /// Espeja `IComponentHandler::restart_component(flags)`: el plugin le
    /// pide al host que vuelva a leer buses/latencia/parámetros según los
    /// bits de `vst3_sys::vst::RestartFlags`.
    RestartComponent(i32),
}

/// Rutas de búsqueda estándar de plugins VST/VST3 en Linux.
/// Se usan únicamente para poblar `Config::vst_paths` en el primer arranque;
/// a partir de ahí el usuario las gestiona (agregar/quitar/reordenar) como cualquier otra ruta.
pub fn default_vst_paths() -> Vec<Arc<Path>> {
    let mut paths = Vec::new();

    if cfg!(unix) {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            paths.push(home.join(".vst").into());
            paths.push(home.join(".vst3").into());
        }

        paths.push(Path::new("/usr/lib/vst").into());
        paths.push(Path::new("/usr/lib/vst3").into());
        paths.push(Path::new("/usr/local/lib/vst3").into());
    }

    if let Some(vst_path) = std::env::var_os("VST3_PATH") {
        paths.extend(std::env::split_paths(&vst_path).map(Arc::from));
    }

    paths
}
