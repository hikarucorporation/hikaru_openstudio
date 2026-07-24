// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use crate::{module::Module, plugin_descriptor::PluginDescriptor};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recorre las rutas configuradas buscando bundles `.vst3` (carpetas, no
/// archivos sueltos: en Linux un plugin VST3 es un directorio con la forma
/// `Nombre.vst3/Contents/x86_64-linux/*.so`), carga cada `.so` real y
/// enumera sus clases de audio-effect vía `factory::enumerate_audio_effects`.
///
/// Igual que `generic_daw_core::clap_host::get_installed_plugins`, nunca
/// devuelve `Result`: los fallos de un plugin individual (bundle corrupto,
/// símbolo faltante, `initialize()` fallido) se loguean con `log::warn!` y
/// simplemente se saltea ese plugin, para que uno roto no tumbe el escaneo
/// completo de todos los demás.
pub fn get_installed_plugins<'a>(
    paths: impl IntoIterator<Item = &'a std::sync::Arc<Path>>,
    mut callback: impl FnMut(PluginDescriptor),
) {
    for base in paths {
        for bundle in find_vst3_bundles(base) {
            let Some(so_path) = find_linux_binary(&bundle) else {
                log::warn!(
                    "{}: no se encontró un .so en Contents/x86_64-linux/",
                    bundle.display()
                );
                continue;
            };

            let module = match Module::load(&so_path) {
                Ok(module) => module,
                Err(err) => {
                    log::warn!("{}: {err}", so_path.display());
                    continue;
                }
            };

            if let Err(err) = crate::factory::enumerate_audio_effects(&module, &mut callback) {
                log::warn!("{}: {err}", so_path.display());
            }
        }
    }
}

/// Encuentra carpetas `*.vst3` bajo `root` (recursivo, un bundle puede estar
/// anidado en subcarpetas de organización del usuario).
fn find_vst3_bundles(root: &Path) -> Vec<PathBuf> {
    if !root.is_dir() {
        return Vec::new();
    }

    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_dir()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("vst3"))
        })
        .map(|entry| entry.into_path())
        .collect()
}

/// Dentro de un bundle `Nombre.vst3/`, ubica el binario real en
/// `Contents/x86_64-linux/*.so`. No asumimos el nombre exacto del .so
/// (no siempre coincide con el nombre del bundle), así que tomamos el
/// primer `.so` que aparezca en esa carpeta.
fn find_linux_binary(bundle: &Path) -> Option<PathBuf> {
    let linux_dir = bundle.join("Contents").join("x86_64-linux");
    if !linux_dir.is_dir() {
        return None;
    }

    WalkDir::new(&linux_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("so"))
        })
        .map(|entry| entry.into_path())
}
