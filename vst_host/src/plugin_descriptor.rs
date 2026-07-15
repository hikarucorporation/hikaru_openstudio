/// Metadata de una clase de plugin VST3, análoga a
/// `clap_host::PluginDescriptor`.
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
	pub class_id: [i8; 16],
	pub name: String,
	pub vendor: String,
	pub category: String,
	pub version: String,
	pub module_path: std::path::PathBuf,
}

impl PluginDescriptor {
	pub(crate) fn class_id_hex(&self) -> String {
		self.class_id
			.iter()
			.map(|b| format!("{:02x}", *b as u8))
			.collect()
	}
}
