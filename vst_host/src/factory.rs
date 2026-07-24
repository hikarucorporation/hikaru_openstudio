// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use crate::module::Module;
use crate::plugin_descriptor::PluginDescriptor;
use std::ffi::c_void;
use vst3_sys::base::{PClassInfo, PClassInfo2, kResultOk};

// Función helper para convertir arrays de i8 (C-string) de ancho fijo a String de Rust
fn i8_array_to_string(arr: &[i8]) -> String {
    let bytes: Vec<u8> = arr.iter()
        .map(|&c| c as u8)
        .take_while(|&b| b != 0)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

// --- REPRESENTACIÓN EXACTA DE LA ABI COM (C++) ---
// Mapeamos las vtables de manera binaria para no depender de las macros de vst3-sys.

#[repr(C)]
struct IUnknownVtbl {
    query_interface: unsafe extern "system" fn(
        this: *mut *const IUnknownVtbl,
        iid: *const c_void,
        obj: *mut *mut c_void,
    ) -> i32,
    add_ref: unsafe extern "system" fn(this: *mut *const IUnknownVtbl) -> u32,
    release: unsafe extern "system" fn(this: *mut *const IUnknownVtbl) -> u32,
}

#[repr(C)]
struct IPluginFactoryVtbl {
    query_interface: unsafe extern "system" fn(this: *mut *const IPluginFactoryVtbl, iid: *const c_void, obj: *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(this: *mut *const IPluginFactoryVtbl) -> u32,
    release: unsafe extern "system" fn(this: *mut *const IPluginFactoryVtbl) -> u32,
    get_factory_info: *const c_void,
    count_classes: unsafe extern "system" fn(this: *mut *const IPluginFactoryVtbl) -> i32,
    get_class_info: unsafe extern "system" fn(
        this: *mut *const IPluginFactoryVtbl,
        index: i32,
        info: *mut PClassInfo,
    ) -> i32,
    create_instance: unsafe extern "system" fn(
        this: *mut *const IPluginFactoryVtbl,
        cid: *const i8,
        iid: *const c_void,
        obj: *mut *mut c_void,
    ) -> i32,
}

#[repr(C)]
struct IPluginFactory2Vtbl {
    query_interface: unsafe extern "system" fn(this: *mut *const IPluginFactory2Vtbl, iid: *const c_void, obj: *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(this: *mut *const IPluginFactory2Vtbl) -> u32,
    release: unsafe extern "system" fn(this: *mut *const IPluginFactory2Vtbl) -> u32,
    get_factory_info: *const c_void,
    count_classes: unsafe extern "system" fn(this: *mut *const IPluginFactory2Vtbl) -> i32,
    get_class_info: unsafe extern "system" fn(this: *mut *const IPluginFactory2Vtbl, index: i32, info: *mut PClassInfo) -> i32,
    create_instance: unsafe extern "system" fn(
        this: *mut *const IPluginFactoryVtbl,
        cid: *const i8,
        iid: *const c_void,
        obj: *mut *mut c_void,
    ) -> i32,
    get_class_info2: unsafe extern "system" fn(
        this: *mut *const IPluginFactory2Vtbl,
        index: i32,
        info: *mut PClassInfo2,
    ) -> i32,
}

// IIDs binarios en formato GUID de C++ para consultar en tiempo de ejecución
const IID_IUNKNOWN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];

const IID_IPLUGINFACTORY2: [u8; 16] = [
    0x50, 0xB6, 0x07, 0x00, 0x4B, 0xF2, 0x0B, 0x4C,
    0xA4, 0x64, 0xED, 0xB9, 0xF0, 0x0B, 0x2A, 0xBB,
];

// IID real de IComponent ("E831FF31-F2D5-4301-928E-BBEE25697802" en vst3-sys).
// createInstance() debe pedir ESTA interfaz, no IUnknown: pedir IUnknown puede
// devolver un puntero al vtable "canónico" del objeto COM, que no necesariamente
// coincide en layout binario con el vtable de IComponent si el plugin implementa
// herencia múltiple internamente. Esto era la causa probable de fallos silenciosos
// o crashes al instanciar plugins reales (Vital incluido).
const IID_ICOMPONENT: [u8; 16] = [
    0x31, 0xFF, 0x31, 0xE8, 0xD5, 0xF2, 0x01, 0x43,
    0x92, 0x8E, 0xBB, 0xEE, 0x25, 0x69, 0x78, 0x02,
];

pub fn enumerate_audio_effects<F>(module: &Module, callback: &mut F) -> Result<(), String>
where
    F: FnMut(PluginDescriptor),
{
    let factory_ptr = module.get_factory_ptr();
    if factory_ptr.is_null() {
        return Err("El puntero a la factoría es nulo.".to_string());
    }

    let factory = factory_ptr as *mut *const IPluginFactoryVtbl;
    let mut factory2: *mut *const IPluginFactory2Vtbl = std::ptr::null_mut();

    unsafe {
        let unknown = factory_ptr as *mut *const IUnknownVtbl;
        let query_interface = (**unknown).query_interface;

        (query_interface)(
            unknown,
            IID_IPLUGINFACTORY2.as_ptr() as *const c_void,
            &mut factory2 as *mut *mut *const IPluginFactory2Vtbl as *mut *mut c_void,
        );
    }

    let num_classes = unsafe {
        let count_classes = (**factory).count_classes;
        (count_classes)(factory)
    };
    println!("[DEBUG] El plugin expone {} clases en total.", num_classes);

    for i in 0..num_classes {
        let mut descriptor = PluginDescriptor {
            name: String::new(),
            vendor: String::new(),
            category: String::new(),
            version: String::new(),
            class_id: [0; 16],
            module_path: module.path().to_path_buf(),
        };

        let mut success = false;

        if !factory2.is_null() {
            let mut class_info = unsafe { std::mem::zeroed::<PClassInfo2>() };
            let res = unsafe {
                let get_class_info2 = (**factory2).get_class_info2;
                (get_class_info2)(factory2, i, &mut class_info)
            };
            if res == kResultOk {
                let mut cid_i8 = [0i8; 16];
                for (dst, src) in cid_i8.iter_mut().zip(class_info.cid.data.iter()) {
                    *dst = *src as i8;
                }
                descriptor.class_id = cid_i8;
                descriptor.name = i8_array_to_string(&class_info.name);
                descriptor.category = i8_array_to_string(&class_info.category);
                descriptor.vendor = i8_array_to_string(&class_info.vendor);
                descriptor.version = i8_array_to_string(&class_info.version);
                success = true;
            }
        }

        if !success {
            let mut class_info = unsafe { std::mem::zeroed::<PClassInfo>() };
            let res = unsafe {
                let get_class_info = (**factory).get_class_info;
                (get_class_info)(factory, i, &mut class_info)
            };
            if res == kResultOk {
                let mut cid_i8 = [0i8; 16];
                for (dst, src) in cid_i8.iter_mut().zip(class_info.cid.data.iter()) {
                    *dst = *src as i8;
                }
                descriptor.class_id = cid_i8;
                descriptor.name = i8_array_to_string(&class_info.name);
                descriptor.category = i8_array_to_string(&class_info.category);
                descriptor.vendor = "Unknown (Legacy Factory)".to_string();
                descriptor.version = "1.0.0 (Legacy)".to_string();
            }
        }

        println!(
            "[DEBUG Class {}] Name: '{}' | Category: '{}' | Vendor: '{}'",
            i, descriptor.name, descriptor.category, descriptor.vendor
        );

        callback(descriptor);
    }

    if !factory2.is_null() {
        unsafe {
            let unknown2 = factory2 as *mut *const IUnknownVtbl;
            let release = (**unknown2).release;
            (release)(unknown2);
        }
    }

    Ok(())
}

/// Crea una instancia del plugin usando su class_id binario.
/// Pide explícitamente la interfaz IComponent (no IUnknown): es la interfaz
/// que `plugin.rs` espera recibir para construir su `VstPtr<dyn IComponent>`.
pub fn create_instance(module: &Module, class_id: &[i8; 16]) -> *mut c_void {
    let factory_ptr = module.get_factory_ptr();
    if factory_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let factory = factory_ptr as *mut *const IPluginFactoryVtbl;
    let mut obj: *mut c_void = std::ptr::null_mut();

    unsafe {
        let create_fn = (**factory).create_instance;
        let res = (create_fn)(
            factory,
            class_id.as_ptr(),
            IID_ICOMPONENT.as_ptr() as *const c_void,
            &mut obj as *mut *mut c_void,
        );

        if res == kResultOk {
            obj
        } else {
            std::ptr::null_mut()
        }
    }
}
