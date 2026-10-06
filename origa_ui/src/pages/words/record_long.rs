//! Long-form microphone recording (AU-3): arbitrary-length capture that
//! feeds the chunked transcription pipeline.
//!
//! WebAudio captures mono PCM at a 16 kHz AudioContext (the browser
//! resamples the mic input), 30-second WAV chunks are pre-encoded while
//! recording (memory: ~1 MB per chunk), and hand-off reuses the same
//! transcription path as the file upload. MediaRecorder is deliberately
//! not used: its webm/opus output has no decoder in the stack.

use leptos::prelude::*;
use tracing::info;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use super::audio_chunks::encode_wav_s16_mono;
use super::audio_transcribe::WavChunk;

pub(super) const CHUNK_SECONDS: f64 = 30.0;
const TARGET_RATE: u32 = 16_000;

/// Everything needed to stop a running capture.
pub(super) struct ActiveRecording {
    stream: web_sys::MediaStream,
    context: web_sys::AudioContext,
    processor: web_sys::ScriptProcessorNode,
    source: web_sys::MediaStreamAudioSourceNode,
    _listener: Closure<dyn FnMut(web_sys::AudioProcessingEvent)>,
}

impl Drop for ActiveRecording {
    fn drop(&mut self) {
        let _ = self.source.disconnect();
        let _ = self.processor.disconnect();
        let _ = self.context.close();
        for track in self.stream.get_audio_tracks() {
            let _ = track.dyn_into::<web_sys::MediaStreamTrack>().map(|t| t.stop());
        }
    }
}

/// A live capture session: polls `chunks()` for completed WAV chunks while
/// recording, and `finish()` stops the mic and returns the tail chunk.
pub(super) struct Recorder {
    pub active: ActiveRecording,
    pcm_buffer: Vec<f32>,
    completed: Vec<WavChunk>,
    pub sample_rate: u32,
}

impl Recorder {
    /// Requests the microphone and starts capturing into 30-second chunks.
    pub(super) async fn start() -> Result<Self, String> {
        let navigator = web_sys::window()
            .ok_or("No window")?
            .navigator();
        let media_devices = navigator.media_devices().ok_or("No media devices")?;
        let mut constraints = web_sys::MediaStreamConstraints::new();
        constraints.audio(&JsValue::from_bool(true));
        let stream = media_devices
            .get_user_media_with_constraints(&constraints)
            .map_err(|e| format!("Microphone access failed: {e:?}"))?
            .await
            .map_err(|e| format!("Microphone access denied: {e:?}"))?;

        let context = web_sys::AudioContext::new_with_options(
            &web_sys::AudioContextOptions::new().sample_rate(f32::from(TARGET_RATE)),
        )
        .map_err(|e| format!("AudioContext failed: {e:?}"))?;
        let source = context
            .create_media_stream_source(&stream)
            .map_err(|e| format!("Audio source failed: {e:?}"))?;
        let processor = context
            .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
                4096,
                1,
                1,
            )
            .map_err(|e| format!("Audio processor failed: {e:?}"))?;

        Ok(Self {
            active: ActiveRecording {
                stream,
                context,
                processor,
                source,
                _listener: Closure::wrap(Box::new(|_: web_sys::AudioProcessingEvent| {})),
            },
            pcm_buffer: Vec::new(),
            completed: Vec::new(),
            sample_rate: TARGET_RATE,
        })
    }

    /// Installs the PCM accumulation listener. Called separately so the
    /// recorder object exists first (the closure captures it through
    /// `Rc<RefCell<..>>` in the caller).
    pub(super) fn take_chunk_sink(
        &mut self,
    ) -> impl FnMut(web_sys::AudioProcessingEvent) + 'static {
        let buffer = std::rc::Rc::new(std::cell::RefCell::new(
            std::mem::take(&mut self.pcm_buffer),
        ));
        let completed = std::rc::Rc::new(std::cell::RefCell::new(std::mem::take(
            &mut self.completed,
        )));
        let rate = self.sample_rate;
        move |event: web_sys::AudioProcessingEvent| {
            let input = event.input_buffer();
            let channels = input.number_of_channels();
            let frame = input.get_channel_data_as_f32(0).unwrap_or_default();
            let mono = mix_event_channels(&frame, channels);
            let mut buffer = buffer.borrow_mut();
            buffer.extend(mono);
            let chunk_samples = (CHUNK_SECONDS * f64::from(rate)).ceil() as usize;
            while buffer.len() >= chunk_samples {
                let chunk: Vec<f32> = buffer.drain(..chunk_samples).collect();
                completed.borrow_mut().push(encode_chunk(chunk, rate));
            }
        }
    }

    /// Completed (already encoded) chunks.
    pub(super) fn take_completed_chunks(&mut self) -> Vec<WavChunk> {
        std::mem::take(&mut self.completed)
    }

    /// Stops the capture and encodes the tail into the final chunk.
    pub(super) fn finish(&mut self) -> Vec<WavChunk> {
        let mut chunks = std::mem::take(&mut self.completed);
        let tail = std::mem::take(&mut self.pcm_buffer);
        if !tail.is_empty() {
            chunks.push(encode_chunk(tail, self.sample_rate));
        }
        chunks
    }
}

fn encode_chunk(samples: Vec<f32>, rate: u32) -> WavChunk {
    WavChunk {
        wav_bytes: encode_wav_s16_mono(&samples, rate),
    }
}

fn mix_event_channels(interleaved: &[f32], channels: u32) -> Vec<f32> {
    match channels {
        0 | 1 => interleaved.to_vec(),
        n => interleaved
            .chunks_exact(n as usize)
            .map(|frame| frame.iter().sum::<f32>() / n as f32)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_channels_average_to_mono() {
        assert_eq!(mix_event_channels(&[1.0, 0.0, 0.25, 0.75], 2), vec![0.5, 0.5]);
        assert_eq!(mix_event_channels(&[0.5, 0.25], 1), vec![0.5, 0.25]);
    }
}
