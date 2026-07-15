// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::ffi::c_void;

pub struct Module {
    _library: Library,
    path: PathBuf,
    factory_ptr: *mut c_void,
}

impl Module {
    pub fn load(path: &Path) -> Result<Self, String> {
        unsafe {
            let library = Library::new(path)
                .map_err(|e| format!("Error cargando biblioteca dinámica: {}", e))?;

            // El punto de entrada obligatorio de todo VST3 es 'GetPluginFactory'
            let get_factory: Symbol<'_, unsafe extern "C" fn() -> *mut c_void> = library
                .get(b"GetPluginFactory\0")
                .map_err(|e| format!("No se encontró el símbolo GetPluginFactory: {}", e))?;

            let factory_ptr = get_factory();
            if factory_ptr.is_null() {
                return Err("GetPluginFactory devolvió un puntero nulo.".to_string());
            }

            Ok(Self {
                _library: library,
                path: path.to_path_buf(),
                factory_ptr,
            })
        }
    }

    pub fn get_factory_ptr(&self) -> *mut c_void {
        self.factory_ptr
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// Implementamos Debug manual ya que libloading::Library no lo implementa
impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("path", &self.path)
            .field("factory_ptr", &self.factory_ptr)
            .finish()
    }
}
