// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host

use std::path::Path;

#[derive(Debug)]
pub enum Vst3Error {
    LoadFailed,
    InvalidFactory,
}

impl std::fmt::Display for Vst3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Module {
}

impl Module {
    pub fn load(_path: &Path) -> Result<Self, Vst3Error> {
        let instance = Self {};
        Ok(instance)
    }
}
