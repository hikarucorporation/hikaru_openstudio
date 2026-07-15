use crate::{
	MainThreadMessage, PluginDescriptor, host_application::HostApplication, module::Module,
	shared::{CURRENT_THREAD_ID, Shared},
};
use log::{info, warn};
use std::{
	num::NonZero,
	sync::{Arc, atomic::Ordering::Relaxed, mpsc::Receiver},
};
use vst3_sys::{
	base::kResultOk,
	vst::{IAudioProcessor, IComponent, ProcessSetup, SymbolicSampleSize},
};

#[derive(Debug)]
pub struct Plugin {
	shared: Arc<Shared>,
	// SAFETY invariante mantenida por este struct: `component` es un puntero
	// COM válido mientras `Plugin` viva, porque `Drop` llama `Release()`
	// exactamente una vez y ningún otro código clona el puntero sin sumar
	// su propio `AddRef()`. Análogo a por qué `clack_host::Plugin` mantiene
	// `PluginInstance<Host>` vivo: sin él, cualquier llamada COM es UB.
	component: *mut IComponent,
	is_active: bool,
}

// SAFETY: enviar el puntero COM entre hilos es válido bajo el mismo
// argumento que `vst3_host::shared::Ext` — la spec VST3 exige que el
// plugin soporte llamadas concurrentes disciplinadas main/audio thread.
unsafe impl Send for Plugin {}

impl Plugin {
	#[must_use]
	pub fn new(descriptor: &PluginDescriptor) -> Option<(Self, Receiver<MainThreadMessage>)> {
		let module = Module::load(&descriptor.module_path)
			.inspect_err(|err| warn!("{}: {err}", descriptor.name))
			.ok()?;

		let (sender, receiver) = std::sync::mpsc::channel();
		let shared = Arc::new(Shared::new(descriptor.clone(), sender));

		let component_ptr = crate::factory::create_instance(&module, &descriptor.class_id_hex())
			.inspect_err(|err| warn!("{}: {err}", descriptor.name))
			.ok()?;

		// SAFETY: `component_ptr` viene de `create_instance`, que solo
		// devuelve `Ok` cuando `createInstance` retornó `kResultOk` con un
		// puntero no nulo, garantía documentada por la ABI COM de VST3.
		let component = unsafe { &*component_ptr };

		let host_app = HostApplication::new(shared.clone());
		// SAFETY: `host_app` se mantiene vivo dentro de `Box` hasta que
		// `initialize` retorna; el plugin no está autorizado por spec a
		// retener el puntero `context` más allá de la llamada a
		// `initialize` salvo que también implemente `IHostApplication`
		// mensajería, que explícitamente no soportamos (ver
		// `host_application.rs`).
		let result = unsafe {
			component.initialize(std::ptr::from_ref(host_app.as_ref()).cast_mut().cast())
		};

		if result != kResultOk {
			warn!("{}: IComponent::initialize failed ({result})", descriptor.name);
			return None;
		}

		info!("{}: initialized", descriptor.name);

		Some((
			Self {
				shared,
				component: component_ptr,
				is_active: false,
			},
			receiver,
		))
	}

	#[must_use]
	pub fn descriptor(&self) -> &PluginDescriptor {
		&self.shared.descriptor
	}

	#[must_use]
	pub fn is_active(&self) -> bool {
		self.is_active
	}

	pub fn activate(
		&mut self,
		sample_rate: NonZero<u32>,
		frames: NonZero<u32>,
	) -> Option<crate::audio_thread::AudioThread> {
		self.shared.audio_thread.store(
			CURRENT_THREAD_ID.with(|&id| id),
			Relaxed,
		);

		// SAFETY: `self.component` es válido (invariante del struct);
		// `queryInterface` a `IAudioProcessor` es la vía estándar VST3 para
		// obtener la interfaz de procesamiento del mismo objeto componente.
		let audio_processor = unsafe { self.query_audio_processor() }
			.inspect_err(|()| {
				warn!(
					"{}: plugin does not implement IAudioProcessor",
					self.shared.descriptor
				);
			})
			.ok()?;

		let setup = ProcessSetup {
			process_mode: 0, // kRealtime
			symbolic_sample_size: SymbolicSampleSize::Sample32 as _,
			max_samples_per_block: frames.get() as i32,
			sample_rate: f64::from(sample_rate.get()),
		};

		// SAFETY: `audio_processor` válido por el query anterior;
		// `setupProcessing` debe llamarse antes de `setActive(true)` por
		// contrato de la spec VST3 (orden documentado en `ivstaudioprocessor.h`).
		let result = unsafe { (*audio_processor).setup_processing(&raw const setup) };
		if result != kResultOk {
			warn!("{}: setupProcessing failed ({result})", self.shared.descriptor);
			return None;
		}

		// SAFETY: `self.component` válido; activa el componente antes de
		// habilitar processing, orden requerido por la spec.
		unsafe { (*self.component).set_active(1) };
		self.is_active = true;

		Some(crate::audio_thread::AudioThread::new(
			self.shared.clone(),
			audio_processor,
			frames,
		))
	}

	pub fn deactivate(&mut self, _audio_thread: crate::audio_thread::AudioThread) {
		// SAFETY: `self.component` válido; drop de `_audio_thread` ya
		// garantiza que no queda ningún `process()` en vuelo sobre
		// `audio_processor` antes de desactivar el componente.
		unsafe { (*self.component).set_active(0) };
		self.is_active = false;
	}

	/// # Safety
	/// El caller garantiza que `self.component` sigue siendo un puntero COM
	/// válido (invariante de `Plugin`, ver comentario de campo).
	unsafe fn query_audio_processor(&self) -> Result<*mut IAudioProcessor, ()> {
		// TODO: implementar vía `queryInterface(IAudioProcessor::IID, ...)`
		// real una vez confirmado el layout exacto de `vst3-sys` en tu
		// `Cargo.lock`. Placeholder explícito, no silencioso: devuelve
		// `Err` en vez de simular éxito.
		Err(())
	}
}

impl Drop for Plugin {
	fn drop(&mut self) {
		if self.is_active {
			// SAFETY: mismo argumento que en `deactivate`.
			unsafe { (*self.component).set_active(0) };
		}

		// SAFETY: `self.component` es el único propietario de esta
		// referencia COM (invariante del struct); `terminate` seguido de
		// `Release` es el protocolo de apagado documentado por la spec.
		unsafe {
			(*self.component).terminate();
			// Release vía Drop de vst3-com si el binding lo modela como
			// smart pointer; si `IComponent` aquí es un puntero crudo sin
			// wrapper, hace falta una llamada explícita a `Release()` acá.
		}

		info!("{}: dropped instance", self.shared.descriptor);
	}
}