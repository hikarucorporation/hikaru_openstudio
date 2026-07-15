use crate::shared::Shared;
use std::sync::Arc;
use vst3_sys::VST3;
use vst3_sys::{
	base::{IPluginBase, kResultOk, kResultFalse, tresult},
	vst::IHostApplication,
};

/// Implementación mínima del lado host de `IHostApplication`. Es lo que el
/// plugin recibe en `IComponent::initialize(context)` para poder, a su vez,
/// pedirle interfaces al host (ej. `IPlugInterfaceSupport`).
///
/// Deliberadamente no implementamos `createInstance` de mensajería
/// (mensajes VST3 arbitrarios entre plugin↔host) en este primer corte:
/// ningún plugin de efecto/instrumento común lo requiere para audio básico.
/// Si un plugin específico lo necesita, hay que sumarlo explícitamente acá.
#[VST3(implements(IHostApplication))]
pub struct HostApplication {
	shared: Arc<Shared>,
}

impl HostApplication {
	pub fn new(shared: Arc<Shared>) -> Box<Self> {
		Self::allocate(shared)
	}
}

impl IHostApplication for HostApplication {
	unsafe fn get_name(&self, name: *mut vst3_sys::base::String128) -> tresult {
		const HOST_NAME: &str = "Hikaru OpenStudio";

		// SAFETY: `name` es un buffer `String128` (128 UTF-16 code units)
		// provisto por el plugin según el contrato de la interfaz; escribir
		// como máximo 127 unidades + null-terminator no lo desborda.
		unsafe {
			let buf = std::slice::from_raw_parts_mut((*name).as_mut_ptr(), 128);
			let encoded: Vec<u16> = HOST_NAME.encode_utf16().take(127).collect();
			buf[..encoded.len()].copy_from_slice(&encoded);
			buf[encoded.len()] = 0;
		}

		kResultOk
	}

	unsafe fn create_instance(
		&self,
		_cid: *mut GUID,
		_iid: *mut GUID,
		_obj: *mut *mut std::ffi::c_void,
	) -> tresult {
		// No soportado en este corte inicial — ver doc-comment de arriba.
		kResultFalse
	}
}