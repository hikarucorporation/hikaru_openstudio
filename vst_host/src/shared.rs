use crate::{MainThreadMessage, PluginDescriptor};
use std::sync::{
	OnceLock,
	atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
	mpsc::Sender,
};
pub struct NoDebug<T>(pub T);
impl<T> std::fmt::Debug for NoDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pointer")
    }
}
impl<T> std::ops::Deref for NoDebug<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
use vst3_sys::vst::{IAudioProcessor, IComponent, IEditController};

static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(0);
thread_local! {
	pub static CURRENT_THREAD_ID: u64 = NEXT_THREAD_ID.fetch_add(1, Relaxed);
}

/// Punteros COM cacheados tras `queryInterface`, análogo a `clap_host::Ext`.
///
/// A diferencia de CLAP (que declara extensiones por trait), en VST3 las
/// "extensiones" son interfaces que se piden vía `queryInterface` sobre el
/// mismo objeto `IComponent`. Se cachean para no repetir el costo (y el
/// riesgo de un `queryInterface` fallido a mitad de audio processing).
#[derive(Debug, Default)]
pub struct Ext {
	pub audio_processor: OnceLock<NoDebug<*mut dyn IAudioProcessor>>,
	pub edit_controller: OnceLock<NoDebug<*mut dyn IEditController>>,
}

// SAFETY: los punteros COM crudos en `Ext` solo se leen tras `queryInterface`
// exitoso y nunca se liberan desde aquí (el `Drop` de `Plugin` los libera vía
// `Release()`). Enviar el puntero entre hilos es seguro porque el contrato
// COM de VST3 exige que la implementación del plugin sea internamente
// thread-safe para las llamadas que el host hace desde audio thread vs. main
// thread — la misma suposición que ya hace `clack_host` para sus extensiones.
unsafe impl Send for Ext {}
unsafe impl Sync for Ext {}

#[derive(Debug)]
pub struct Shared {
	pub descriptor: PluginDescriptor,
	pub sender: Sender<MainThreadMessage>,
	pub ext: Ext,
	pub main_thread: u64,
	pub audio_thread: AtomicU64,
	/// Mirror de `clap_host::Shared::request_restart`: seteado desde el
	/// callback `IComponentHandler::restartComponent`.
	pub request_restart: AtomicBool,
}

impl Shared {
	pub fn new(descriptor: PluginDescriptor, sender: Sender<MainThreadMessage>) -> Self {
		let main_thread = CURRENT_THREAD_ID.with(|&id| id);

		Self {
			descriptor,
			sender,
			ext: Ext::default(),
			main_thread,
			audio_thread: AtomicU64::new(main_thread),
			request_restart: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn is_main_thread(&self) -> bool {
		CURRENT_THREAD_ID.with(|&id| id == self.main_thread)
	}

	#[must_use]
	pub fn is_audio_thread(&self) -> bool {
		CURRENT_THREAD_ID.with(|&id| id == self.audio_thread.load(Relaxed))
	}
}