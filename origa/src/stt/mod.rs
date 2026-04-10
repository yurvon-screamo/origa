mod audio;
mod common;
mod mel_spectrogram;
mod tokenizer;

#[cfg(not(target_arch = "wasm32"))]
mod whisper;

#[cfg(target_arch = "wasm32")]
mod whisper_wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use whisper::WhisperTranscriber;

#[cfg(target_arch = "wasm32")]
pub use whisper_wasm::WhisperTranscriber;

#[cfg(target_arch = "wasm32")]
pub use audio::load_audio_bytes;

#[cfg(test)]
mod audio_tests;
#[cfg(test)]
mod tests;
