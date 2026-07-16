// SPDX-License-Identifier: LGPL-3.0-or-later
use log::{warn, info};
use std::ffi::c_void;
use std::sync::Arc;
use std::fmt;
use vst3_sys::base::{kResultOk, IUnknown};
use vst3_sys::vst::{IAudioProcessor, IComponent, ProcessSetup, SymbolicSampleSizes};
use vst3_sys::{ComInterface, VstPtr};
use crate::module::Module;

type Tuid = [i8; 16];

pub struct PluginDescriptor {
    pub name: String,
    pub class_id: Tuid,
}

impl fmt::Debug for PluginDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginDescriptor").field("name", &self.name).finish()
    }
}

pub struct Plugin {
    pub descriptor: Arc<PluginDescriptor>,
    pub component: VstPtr<dyn IComponent>,
    pub processor: Option<VstPtr<dyn IAudioProcessor>>,
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plugin").field("name", &self.descriptor.name).finish()
    }
}

impl Plugin {
    pub fn load(module: &Module, descriptor: Arc<PluginDescriptor>) -> Result<Self, String> {
        info!("Cargando plugin: {}", descriptor.name);
        let component_ptr = crate::factory::create_instance(module, &descriptor.class_id);
        if component_ptr.is_null() {
            return Err(format!("{}: No se pudo instanciar el componente", descriptor.name));
        }

        let component: VstPtr<dyn IComponent> = unsafe {
            VstPtr::owned(component_ptr as *mut *mut _)
                .ok_or_else(|| format!("{}: Error al instanciar VstPtr para el componente", descriptor.name))?
        };

        let mut processor = None;
        let mut iid = <dyn IAudioProcessor>::IID;
        let mut obj: *mut c_void = std::ptr::null_mut();

        let res = unsafe { component.query_interface(&mut iid, &mut obj) };

        if res == kResultOk && !obj.is_null() {
            let proc_ptr: VstPtr<dyn IAudioProcessor> = unsafe {
                VstPtr::owned(obj as *mut *mut _)
                    .ok_or_else(|| format!("{}: Error al instanciar VstPtr para el procesador", descriptor.name))?
            };
            processor = Some(proc_ptr);
            info!("Procesador de audio enlazado con éxito para {}", descriptor.name);
        } else {
            warn!("{}: El componente no expone la interfaz de procesamiento de audio", descriptor.name);
        }

        Ok(Self { descriptor, component, processor })
    }

    pub fn prepare_to_play(&self, sample_rate: f64, max_samples_per_block: i32) -> Result<(), String> {
        if let Some(ref processor) = self.processor {
            let mut setup = ProcessSetup {
                process_mode: 0,
                symbolic_sample_size: SymbolicSampleSizes::kSample32 as _,
                max_samples_per_block,
                sample_rate,
            };

            let res = unsafe { processor.setup_processing(&mut setup) };
            if res != kResultOk {
                return Err(format!("Fallo setup_processing en {}: {}", self.descriptor.name, res));
            }
        }
        Ok(())
    }
}
