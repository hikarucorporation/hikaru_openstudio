use cpal::{SampleFormat, SampleRate};
use std::num::NonZeroU32;

// Ajuste para el sample_rate usando NonZeroU32 nativo y acceso al valor .0
pub fn get_sample_rate(config: &cpal::SupportedStreamConfig) -> NonZeroU32 {
    let rate_u32 = config.sample_rate().0;
    NonZeroU32::new(rate_u32).expect("Sample rate must be non-zero")
}

// Para el clamp, operamos sobre los u32 internos
pub fn clamp_rate(sample_rate: SampleRate, min: SampleRate, max: SampleRate) -> SampleRate {
    let rate = sample_rate.0.clamp(min.0, max.0);
    SampleRate(rate)
}

// Ajuste para el match de SampleFormat (usamos las variantes que tu cpal reconoce)
pub fn format_to_internal(format: SampleFormat) -> String {
    match format {
        SampleFormat::I8 => "I8".to_string(),
        SampleFormat::I16 => "I16".to_string(),
        SampleFormat::I32 => "I32".to_string(),
        SampleFormat::I64 => "I64".to_string(),
        SampleFormat::U8 => "U8".to_string(),
        SampleFormat::U16 => "U16".to_string(),
        SampleFormat::U32 => "U32".to_string(),
        SampleFormat::U64 => "U64".to_string(),
        SampleFormat::F32 => "F32".to_string(),
        SampleFormat::F64 => "F64".to_string(),
        _ => "Unknown".to_string(),
    }
}
