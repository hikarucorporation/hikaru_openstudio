// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host
//
// Implementación de `IComponentHandler` (GUID real del SDK:
// 93A0BEA3-0BD0-45DB-8E89-0B0CC1E46AC6, confirmado en vst3-sys pineado
// f3e8f01c3de6d5df2f503c920c9f2bf8166a771b, src/vst/ivsteditcontroller.rs).
//
// Es la única vía plugin -> host en VST3 para: automatización de parámetros
// (begin_edit/perform_edit/end_edit) y reconfiguración en caliente
// (restart_component). El plugin recibe un puntero a esto vía
// `IEditController::set_component_handler` y lo llama desde su propio hilo
// (normalmente el main thread del host, pero el contrato COM no lo garantiza).

use crate::shared::Shared;
use crate::MainThreadMessage;
use log::{debug, warn};
use std::ffi::c_void;
use std::sync::Arc;
use vst3_sys::base::{kResultFalse, kResultOk, tresult};

type ParamId = u32;
type ParamValue = f64;

const IID_IUNKNOWN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];

// GUID real de IComponentHandler, confirmado contra vst3-sys pineado.
const IID_ICOMPONENTHANDLER: [u8; 16] = [
    0x93, 0xA0, 0xBE, 0xA3, 0x0B, 0xD0, 0x45, 0xDB,
    0x8E, 0x89, 0x0B, 0x0C, 0xC1, 0xE4, 0x6A, 0xC6,
];

#[repr(C)]
struct IComponentHandlerVtbl {
    // IUnknown
    query_interface: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl, iid: *const c_void, obj: *mut *mut c_void) -> tresult,
    add_ref: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl) -> u32,
    release: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl) -> u32,
    // IComponentHandler
    begin_edit: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl, id: ParamId) -> tresult,
    perform_edit: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl, id: ParamId, value_normalized: ParamValue) -> tresult,
    end_edit: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl, id: ParamId) -> tresult,
    restart_component: unsafe extern "system" fn(this: *mut *const IComponentHandlerVtbl, flags: i32) -> tresult,
}

#[repr(C)]
pub struct ComponentHandler {
    vtable: *const IComponentHandlerVtbl,
    shared: Arc<Shared>,
    ref_count: std::sync::atomic::AtomicU32,
}

static COMPONENT_HANDLER_VTBL: IComponentHandlerVtbl = IComponentHandlerVtbl {
    query_interface,
    add_ref,
    release,
    begin_edit,
    perform_edit,
    end_edit,
    restart_component,
};

impl ComponentHandler {
    pub fn new(shared: Arc<Shared>) -> Box<Self> {
        Box::new(ComponentHandler {
            vtable: &COMPONENT_HANDLER_VTBL,
            shared,
            ref_count: std::sync::atomic::AtomicU32::new(1),
        })
    }
}

unsafe extern "system" fn query_interface(
    this: *mut *const IComponentHandlerVtbl,
    iid: *const c_void,
    obj: *mut *mut c_void,
) -> tresult {
    if iid.is_null() || obj.is_null() {
        return kResultFalse;
    }

    unsafe {
        let iid_slice = std::slice::from_raw_parts(iid as *const u8, 16);
        if iid_slice == IID_IUNKNOWN || iid_slice == IID_ICOMPONENTHANDLER {
            add_ref(this);
            *obj = this as *mut c_void;
            kResultOk
        } else {
            *obj = std::ptr::null_mut();
            kResultFalse
        }
    }
}

unsafe extern "system" fn add_ref(this: *mut *const IComponentHandlerVtbl) -> u32 {
    unsafe {
        let handler = &*(this as *const ComponentHandler);
        handler.ref_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }
}

unsafe extern "system" fn release(this: *mut *const IComponentHandlerVtbl) -> u32 {
    unsafe {
        let handler = &*(this as *const ComponentHandler);
        let prev = handler.ref_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if prev == 1 {
            let _ = Box::from_raw(this as *mut ComponentHandler);
            0
        } else {
            prev - 1
        }
    }
}

// begin_edit/perform_edit/end_edit: todavía no hay UI de automatización de
// parámetros del lado GUI que consuma esto, así que deliberadamente no se
// agranda `MainThreadMessage` con variantes sin consumidor (misma regla que
// ya se aplicó al resto del enum). Solo logueamos para poder confirmar en
// runtime que el plugin efectivamente llama estos callbacks, y se cablean a
// mensajes reales cuando exista la contraparte de UI.

unsafe extern "system" fn begin_edit(this: *mut *const IComponentHandlerVtbl, id: ParamId) -> tresult {
    let handler = unsafe { &*(this as *const ComponentHandler) };
    debug!("{}: IComponentHandler::begin_edit({id})", handler.shared.descriptor.name);
    kResultOk
}

unsafe extern "system" fn perform_edit(
    this: *mut *const IComponentHandlerVtbl,
    id: ParamId,
    value_normalized: ParamValue,
) -> tresult {
    let handler = unsafe { &*(this as *const ComponentHandler) };
    debug!(
        "{}: IComponentHandler::perform_edit({id}, {value_normalized})",
        handler.shared.descriptor.name
    );
    kResultOk
}

unsafe extern "system" fn end_edit(this: *mut *const IComponentHandlerVtbl, id: ParamId) -> tresult {
    let handler = unsafe { &*(this as *const ComponentHandler) };
    debug!("{}: IComponentHandler::end_edit({id})", handler.shared.descriptor.name);
    kResultOk
}

unsafe extern "system" fn restart_component(
    this: *mut *const IComponentHandlerVtbl,
    flags: i32,
) -> tresult {
    let handler = unsafe { &*(this as *const ComponentHandler) };

    handler.shared.request_restart.store(true, std::sync::atomic::Ordering::Relaxed);

    if handler.shared.sender.send(MainThreadMessage::RestartComponent(flags)).is_err() {
        warn!(
            "{}: restart_component({flags}) descartado — receptor MainThreadMessage cerrado",
            handler.shared.descriptor.name
        );
    }

    kResultOk
}
