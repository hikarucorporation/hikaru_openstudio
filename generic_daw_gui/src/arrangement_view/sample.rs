use crate::{arrangement_view::crc, lod::Lods};
use generic_daw_core::{SampleId, Transport, time::SecondsTime};
use std::{fs::File, num::NonZero, path::Path, sync::Arc};
use utils::NoDebug;

#[derive(Clone, Debug)]
pub struct Sample {
	pub id: SampleId,
	pub lods: Lods,
	pub name: Arc<str>,
	pub path: Arc<Path>,
	pub samples: NoDebug<Arc<[[f32; 2]]>>,
	#[expect(clippy::struct_field_names)]
	pub sample_rate: NonZero<u32>,
	pub crc: u32,
	pub len: u64,
	pub refs: usize,
}

impl Sample {
	pub fn resample_ratio(&self, transport: &Transport) -> f64 {
		f64::from(transport.sample_rate.get()) / f64::from(self.sample_rate.get())
	}

	pub fn len(&self, transport: &Transport) -> SecondsTime {
		SecondsTime::from_frames(self.samples.len(), transport) * self.resample_ratio(transport)
	}
}

#[derive(Clone, Debug)]
pub struct SamplePair {
	pub core: generic_daw_core::Sample,
	pub gui: Sample,
}

impl SamplePair {
	pub fn from_core_with_crc_and_len(
		core: generic_daw_core::Sample,
		crc: u32,
		len: u64,
		path: Arc<Path>,
	) -> Option<Self> {
		let lods = Lods::new(&core.data); 
		Self::from_core_and_lods_with_crc_and_len(core, lods, crc, len, path)
	}

	pub fn from_core_and_lods_with_crc_and_len(
		core: generic_daw_core::Sample,
		lods: Lods,
		crc: u32,
		len: u64,
		path: Arc<Path>,
	) -> Option<Self> {
		let name = path.file_name()?.to_str()?.into();
		let gui = Sample {
			id: core.id,
			lods,
			path,
			name,
			samples: core.data.clone(),
			sample_rate: std::num::NonZero::new(core.sample_rate)
				.unwrap_or_else(|| std::num::NonZero::new(44100).unwrap()),
			crc,
			len,
			refs: 0,
		};
		Some(Self { core, gui })
	}
}
