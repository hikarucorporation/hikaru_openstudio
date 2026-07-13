// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use vst3::Steinberg::TUID;

pub struct PClassInfo {
    pub cid: TUID,
    pub category: [i8; 32],
}

pub struct PluginDescriptor {
    pub class_id: [i8; 16],
}

pub fn check_factory_info(info: &PClassInfo) -> Option<PluginDescriptor> {
    let category_bytes: Vec<u8> = info.category.iter()
        .map(|&b| b as u8)
        .take_while(|&b| b != 0)
        .collect();

    if let Ok(category_str) = std::str::from_utf8(&category_bytes) {
        if category_str == "Audio Effect" {
            return Some(PluginDescriptor {
                class_id: info.cid,
            });
        }
    }

    None
}

pub fn enumerate_audio_effects<F, P>(_module: &super::module::Module, _f: &mut F) -> Result<(), super::module::Vst3Error> 
where
    F: FnMut(P),
{
    Ok(())
}
