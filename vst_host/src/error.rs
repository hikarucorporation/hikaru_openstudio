use std::{fmt, io, path::PathBuf};

/// Errores de carga y descubrimiento de plugins VST3.
///
/// Deliberadamente no implementa `Copy`: `Io` y `MissingSymbol` cargan un
/// `PathBuf`/`String` heap-allocated. Esto es correcto porque estos errores
/// solo se producen en el hilo de escaneo/carga (main thread), nunca en el
/// audio thread.
#[derive(Debug)]
pub enum Vst3Error {
	/// Falló abrir el `.so`/bundle en disco.
	Io { path: PathBuf, source: io::Error },
	/// El módulo cargó pero no exporta `GetPluginFactory`.
	MissingSymbol { path: PathBuf, symbol: &'static str },
	/// `GetPluginFactory` devolvió null.
	NullFactory { path: PathBuf },
	/// La clase pedida no es de categoría `kVstAudioEffectClass`.
	NotAnAudioEffect { class_id: String },
	/// `createInstance` devolvió un código de error COM distinto de `kResultOk`.
	InstantiationFailed { class_id: String, code: i32 },
}

impl fmt::Display for Vst3Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
			Self::MissingSymbol { path, symbol } => {
				write!(f, "{}: missing symbol `{symbol}`", path.display())
			}
			Self::NullFactory { path } => {
				write!(f, "{}: GetPluginFactory returned null", path.display())
			}
			Self::NotAnAudioEffect { class_id } => {
				write!(f, "class {class_id} is not a kVstAudioEffectClass")
			}
			Self::InstantiationFailed { class_id, code } => {
				write!(f, "createInstance({class_id}) failed with code {code}")
			}
		}
	}
}

impl std::error::Error for Vst3Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Io { source, .. } => Some(source),
			_ => None,
		}
	}
}
