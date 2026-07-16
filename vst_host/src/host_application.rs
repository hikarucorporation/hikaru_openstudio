// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use crate::shared::Shared;
use std::sync::Arc;
use std::ffi::c_void;
use vst3_sys::base::{kResultOk, kResultFalse, tresult};

// Definimos el alias local idéntico a la representación del SDK
type Tuid = [i8; 16];

// --- REPRESENTACIÓN BINARIA COM PARA IHostApplication ---

#[repr(C)]
struct IHostApplicationVtbl {
    // IUnknown
    query_interface: unsafe extern "system" fn(this: *mut *const IHostApplicationVtbl, iid: *const c_void, obj: *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(this: *mut *const IHostApplicationVtbl) -> u32,
    release: unsafe extern "system" fn(this: *mut *const IHostApplicationVtbl) -> u32,
    // IHostApplication
    get_name: unsafe extern "system" fn(this: *mut *const IHostApplicationVtbl, name: *mut vst3_sys::base::tchar) -> tresult,
    create_instance: unsafe extern "system" fn(
        this: *mut *const IHostApplicationVtbl,
        cid: *const Tuid,
        iid: *const Tuid,
        obj: *mut *mut c_void,
    ) -> tresult,
}

#[repr(C)]
pub struct HostApplication {
    vtable: *const IHostApplicationVtbl,
    shared: Arc<Shared>,
    ref_count: std::sync::atomic::AtomicU32,
}

// Interfaz UID de IHostApplication e IUnknown para responder a QueryInterface
const IID_IUNKNOWN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];

const IID_IHOSTAPPLICATION: [u8; 16] = [
    0x58, 0x22, 0xFA, 0x58, 0x0B, 0xAB, 0x4B, 0x2E,
    0xB8, 0x11, 0xEB, 0xFA, 0x24, 0x56, 0x93, 0x96,
];

static HOST_APP_VTBL: IHostApplicationVtbl = IHostApplicationVtbl {
    query_interface,
    add_ref,
    release,
    get_name,
    create_instance,
};

impl HostApplication {
    pub fn new(shared: Arc<Shared>) -> Box<Self> {
        Box::new(HostApplication {
            vtable: &HOST_APP_VTBL,
            shared,
            ref_count: std::sync::atomic::AtomicU32::new(1),
        })
    }
}

// --- IMPLEMENTACIÓN DE LOS MÉTODOS COM ---

unsafe extern "system" fn query_interface(
    this: *mut *const IHostApplicationVtbl,
    iid: *const c_void,
    obj: *mut *mut c_void,
) -> i32 {
    if iid.is_null() || obj.is_null() {
        return kResultFalse;
    }

    unsafe {
        let iid_slice = std::slice::from_raw_parts(iid as *const u8, 16);
        if iid_slice == IID_IUNKNOWN || iid_slice == IID_IHOSTAPPLICATION {
            add_ref(this);
            *obj = this as *mut c_void;
            kResultOk
        } else {
            *obj = std::ptr::null_mut();
            kResultFalse
        }
    }
}

unsafe extern "system" fn add_ref(this: *mut *const IHostApplicationVtbl) -> u32 {
    unsafe {
        let host_app = &*(this as *const HostApplication);
        host_app.ref_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }
}

unsafe extern "system" fn release(this: *mut *const IHostApplicationVtbl) -> u32 {
    unsafe {
        let host_app = &*(this as *const HostApplication);
        let prev = host_app.ref_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if prev == 1 {
            let _ = Box::from_raw(this as *mut HostApplication);
            0
        } else {
            prev - 1
        }
    }
}

unsafe extern "system" fn get_name(
    _this: *mut *const IHostApplicationVtbl,
    name: *mut vst3_sys::base::tchar,
) -> tresult {
    const HOST_NAME: &str = "Hikaru OpenStudio";

    if name.is_null() {
        return kResultFalse;
    }

    unsafe {
        let buf = std::slice::from_raw_parts_mut(name, 128);
        let encoded: Vec<i16> = HOST_NAME.encode_utf16().take(127).map(|c| c as i16).collect();
        buf[..encoded.len()].copy_from_slice(&encoded);
        buf[encoded.len()] = 0;
    }

    kResultOk
}

unsafe extern "system" fn create_instance(
    _this: *mut *const IHostApplicationVtbl,
    _cid: *const Tuid,
    _iid: *const Tuid,
    _obj: *mut *mut c_void,
) -> tresult {
    kResultFalse
}
