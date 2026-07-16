// SPDX-License-Identifier: LGPL-3.0-or-later
use std::sync::Arc;
use std::fmt;
use vst3_sys::vst::{IAudioProcessor, ProcessData};
use crate::shared::Shared;

// Envolvemos el puntero crudo para marcarlo como Send
struct SafeProcessor(*mut dyn IAudioProcessor);
unsafe impl Send for SafeProcessor {}

pub struct AudioThread {
    shared: Arc<Shared>,
    audio_processor: SafeProcessor,
}

impl fmt::Debug for AudioThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioThread").finish()
    }
}

impl AudioThread {
    pub fn new(shared: Arc<Shared>, processor: *mut dyn IAudioProcessor) -> Self {
        Self { shared, audio_processor: SafeProcessor(processor) }
    }

    pub unsafe fn process(&mut self, data: &mut ProcessData) -> i32 {
        if !self.audio_processor.0.is_null() {
            unsafe { (*self.audio_processor.0).process(data) }
        } else {
            0
        }
    }
}
