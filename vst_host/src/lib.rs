// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

pub mod module;
pub mod factory;
pub mod plugin_descriptor;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use crate::plugin_descriptor::PluginDescriptor;
use crate::module::Module;
use crate::factory::enumerate_audio_effects;

/// Escanea los directorios VST3 indicados buscando plugins válidos y ejecuta un callback por cada uno.
pub fn get_installed_plugins<F>(vst3_paths: &[Arc<Path>], mut callback: F)
where
    F: FnMut(PluginDescriptor),
{
    // Si la GUI nos pasa rutas personalizadas, las usamos; si no, metemos los defaults.
    let paths: Vec<PathBuf> = if vst3_paths.is_empty() {
        let mut defaults = vec![
            PathBuf::from("/usr/lib/vst3"),
            PathBuf::from("/usr/local/lib/vst3"),
        ];
        if let Some(mut home) = std::env::var_os("HOME").map(PathBuf::from) {
            home.push(".vst3");
            defaults.push(home);
        }
        defaults
    } else {
        vst3_paths.iter().map(|p| p.to_path_buf()).collect()
    };

    for base_path in paths {
        if !base_path.exists() {
            continue;
        }

        // Buscamos archivos .so dentro de estructuras de directorios .vst3
        for entry in WalkDir::new(&base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            
            // FILTRO DE SEGURIDAD: Evitamos yabridge para que no reviente el host al intentar cargarlos nativamente.
            if path_str.contains("yabridge") {
                continue;
            }

            // Filtramos librerías dinámicas dentro de bundles .vst3
            if path.is_file() && path.extension().map_or(false, |ext| ext == "so") {
                if path_str.contains(".vst3/") {
                    println!("[DEBUG Scanner] Intentando cargar plugin en: {:?}", path);
                    
                    // Cargamos el módulo dinámico de manera segura
                    if let Ok(module) = Module::load(path) {
                        // Enlistamos y disparamos el callback que viene del frontend
                        let _ = enumerate_audio_effects(&module, &mut callback);
                    }
                }
            }
        }
    }
}
