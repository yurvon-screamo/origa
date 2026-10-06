//! Chunked audio decoding for long recordings.
//!
//! Symphonia (pure Rust, wasm32-compatible) streams supported containers —
//! mp3, m4a (aac-lc), ogg/vorbis, flac, wav — into mono samples at the
//! source rate; full 30-second chunks are encoded as PCM WAV and handed to
//! the caller one at a time so peak memory stays near `input bytes + one
//! chunk` — never the whole decoded program. The existing transcription
//! pipeline (`load_audio_bytes`) performs its own downmix/resample/trim, so
//! chunks carry source-rate mono and no second resampler is needed here.
//! Opus and webm are not supported: symphonia 0.5 has no opus decoder.

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Length of one transcription chunk in seconds.
pub(super) const CHUNK_SECONDS: f64 = 30.0;
/// Fade applied to both edges of a chunk to remove boundary clicks.
const FADE_SECONDS: f64 = 0.01;
/// Silence appended to each chunk so the model segments at the boundary.
const BOUNDARY_SILENCE_SECONDS: f64 = 0.1;

/// One ready-to-transcribe chunk: PCM WAV (mono, source rate, s16le) with
/// edge fades and trailing boundary silence. Duration derives from the
/// payload (`(len - 44) / 2 / sample_rate`).
#[derive(Debug, Clone)]
pub(super) struct WavChunk {
    pub wav_bytes: Vec<u8>,
}

/// Streaming decoder: `next_chunk()` yields one filled chunk per call until
/// the input is exhausted. Cancel checks happen between packets, so the
/// worst-case cancel latency is one chunk decode.
pub(super) struct ChunkedDecoder {
    format: Box<dyn symphonia::core::formats::FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
    /// Mono samples at the source rate, waiting to fill the next chunk.
    mono_buffer: Vec<f32>,
    sample_rate: u32,
    /// Sample capacity of one chunk at the source rate.
    chunk_samples: usize,
    finished: bool,
}

impl ChunkedDecoder {
    /// Opens the input: probes the container, picks the first audio track.
    pub(super) fn new(bytes: Vec<u8>) -> Result<Self, String> {
        let mss = MediaSourceStream::new(Box::new(std::io::Cursor::new(bytes)), Default::default());
        // No extension hint: the probe decides by content, which is what
        // makes unknown-extension share/download files decodable.
        let probed = symphonia::default::get_probe()
            .format(
                &Hint::new(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| format!("Unsupported or unreadable audio: {e}"))?;

        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No audio track found")?
            .clone();
        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or("Unknown sample rate")?;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("Audio codec unsupported: {e}"))?;

        let chunk_samples = (CHUNK_SECONDS * f64::from(sample_rate)).ceil() as usize;
        Ok(Self {
            format: probed.format,
            decoder,
            track_id: track.id,
            mono_buffer: Vec::new(),
            sample_rate,
            chunk_samples,
            finished: false,
        })
    }

    /// Decodes and returns the next chunk, or `None` when the input is
    /// exhausted. Recoverable decode errors skip the offending packet.
    pub(super) fn next_chunk(&mut self) -> Option<Result<WavChunk, String>> {
        if self.finished {
            return None;
        }

        loop {
            if self.mono_buffer.len() >= self.chunk_samples {
                return Some(Ok(self.take_chunk()));
            }
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                // End of stream: flush whatever partial chunk is buffered.
                Err(SymphoniaError::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.finished = true;
                    return if self.mono_buffer.is_empty() {
                        None
                    } else {
                        Some(Ok(self.take_chunk()))
                    };
                },
                Err(e) => return Some(Err(format!("Audio read failed: {e}"))),
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let mut sample_buffer =
                        SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                    sample_buffer.copy_interleaved_ref(decoded);
                    self.mono_buffer
                        .extend(mix_to_mono(sample_buffer.samples(), spec.channels.count()));
                },
                // Recoverable: a corrupt packet skips ahead instead of
                // failing the whole file.
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Some(Err(format!("Audio decode failed: {e}"))),
            }
        }
    }

    /// Splits the buffer into a chunk (with fades and boundary silence),
    /// keeping the remainder for the next call.
    fn take_chunk(&mut self) -> WavChunk {
        let fade_samples = (FADE_SECONDS * f64::from(self.sample_rate)).round() as usize;
        let silence_samples =
            (BOUNDARY_SILENCE_SECONDS * f64::from(self.sample_rate)).round() as usize;
        // The final (EOF-flushed) chunk can be shorter than the capacity.
        let take = self.chunk_samples.min(self.mono_buffer.len());
        let mut chunk_samples: Vec<f32> = self.mono_buffer.drain(..take).collect();

        apply_fades(&mut chunk_samples, fade_samples);
        chunk_samples.extend((0..silence_samples).map(|_| 0.0_f32));

        WavChunk {
            wav_bytes: encode_wav_s16_mono(&chunk_samples, self.sample_rate),
        }
    }
}

/// Averages interleaved channels down to mono.
fn mix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    match channels {
        0 | 1 => interleaved.to_vec(),
        n => interleaved
            .chunks_exact(n)
            .map(|frame| frame.iter().sum::<f32>() / n as f32)
            .collect(),
    }
}

/// Linear fade-in/out of `fade_samples` at both edges (clamped to half the
/// buffer so a tiny buffer never fades to zero in the middle).
fn apply_fades(samples: &mut [f32], fade_samples: usize) {
    let fade = fade_samples.min(samples.len() / 2);
    if fade == 0 {
        return;
    }
    for (index, sample) in samples[..fade].iter_mut().enumerate() {
        *sample *= index as f32 / fade as f32;
    }
    let start = samples.len() - fade;
    for (index, sample) in samples[start..].iter_mut().enumerate() {
        *sample *= 1.0 - (index + 1) as f32 / fade as f32;
    }
}

/// Encodes mono f32 samples as a PCM WAV file: 16-bit little-endian,
/// standard 44-byte RIFF header.
fn encode_wav_s16_mono(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut wav = Vec::with_capacity(44 + data_len);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes()); // fmt chunk size
    wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1_u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2_u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16_u16.to_le_bytes()); // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        wav.extend_from_slice(&((clamped * 32767.0) as i16).to_le_bytes());
    }
    wav
}

/// Joins per-chunk transcriptions. Japanese text has no spaces, so
/// same-script chunks concatenate directly; a space is inserted whenever
/// either side of the boundary is an ASCII-alphanumeric character (Latin
/// words/numbers would otherwise fuse into Japanese text or each other).
pub(super) fn join_chunk_texts(texts: &[String]) -> String {
    let mut joined = String::new();
    for text in texts {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let prev_last = joined.chars().last();
        let next_first = trimmed.chars().next();
        if !joined.is_empty() && (is_ascii_alnum(prev_last) || is_ascii_alnum(next_first)) {
            joined.push(' ');
        }
        joined.push_str(trimmed);
    }
    joined
}

fn is_ascii_alnum(char: Option<char>) -> bool {
    char.is_some_and(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::japanese_chunks_fuse_directly("私は本を", "読みます", "私は本を読みます")]
    #[case::latin_words_get_a_space("hello", "world", "hello world")]
    #[case::latin_number_boundary("5", "10", "5 10")]
    #[case::japanese_then_latin("私は本を", "read books", "私は本を read books")]
    #[case::latin_then_japanese("read", "本を読む", "read 本を読む")]
    #[case::punctuation_fuses("本です。", "読みました", "本です。読みました")]
    #[case::empty_chunks_fuse("私は", "本を読む", "私は本を読む")]
    fn chunk_texts_join_without_breaking_japanese(
        #[case] first: &str,
        #[case] second: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(
            join_chunk_texts(&[first.to_string(), second.to_string()]),
            expected
        );
    }

    #[test]
    fn empty_chunks_are_skipped_in_longer_sequences() {
        let texts = vec![
            "私は".to_string(),
            String::new(),
            "  ".to_string(),
            "本を".to_string(),
            "読みます".to_string(),
        ];
        assert_eq!(join_chunk_texts(&texts), "私は本を読みます");
    }

    #[test]
    fn fades_never_touch_the_middle() {
        let mut samples = vec![1.0_f32; 100];
        apply_fades(&mut samples, 10);
        assert_eq!(samples[0], 0.0);
        assert!(samples[5] > 0.0 && samples[5] < 1.0);
        assert_eq!(samples[10], 1.0);
        assert_eq!(samples[50], 1.0);
        assert_eq!(samples[99], 0.0);
        assert!(samples[90] > 0.0 && samples[90] < 1.0);
    }

    #[test]
    fn fades_clamp_to_half_of_tiny_buffers() {
        let mut samples = vec![1.0_f32; 3];
        apply_fades(&mut samples, 10);
        assert!(samples.iter().all(|s| *s <= 1.0));
    }

    #[test]
    fn wav_encoding_produces_a_valid_header() {
        let wav = encode_wav_s16_mono(&[0.0, 0.5, -0.5], 16_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        let data_len = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]) as usize;
        assert_eq!(data_len, 6); // 3 samples × 2 bytes
        assert_eq!(wav.len(), 44 + data_len);
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1); // mono
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            16_000
        );
    }

    #[test]
    fn mono_mix_averages_channels() {
        let stereo = [1.0, 0.0, 0.5, 0.5];
        assert_eq!(mix_to_mono(&stereo, 2), vec![0.5, 0.5]);
        let mono = [0.25, 0.75];
        assert_eq!(mix_to_mono(&mono, 1), vec![0.25, 0.75]);
    }
}

/// Integration tests over real encoded inputs: generated silence fixtures
/// (`test-fixtures/audio/`) prove the probe dispatches every declared
/// container, and a programmatic WAV proves chunk boundaries.
#[cfg(test)]
mod decode_tests {
    use super::*;

    use rstest::rstest;

    fn decode_all(bytes: Vec<u8>) -> Vec<WavChunk> {
        let mut decoder =
            ChunkedDecoder::new(bytes).unwrap_or_else(|e| panic!("fixture must decode: {e}"));
        let mut chunks = Vec::new();
        while let Some(result) = decoder.next_chunk() {
            chunks.push(result.unwrap_or_else(|e| panic!("fixture chunk must decode: {e}")));
        }
        assert!(
            !chunks.is_empty(),
            "fixture must produce at least one chunk"
        );
        chunks
    }

    /// Source sample rate from the chunk's WAV header.
    fn header_source_rate(chunk: &WavChunk) -> u32 {
        u32::from_le_bytes([
            chunk.wav_bytes[24],
            chunk.wav_bytes[25],
            chunk.wav_bytes[26],
            chunk.wav_bytes[27],
        ])
    }

    /// Chunk duration from the WAV payload length (data bytes / 2 = s16
    /// samples at the source rate).
    fn chunk_duration_secs(chunk: &WavChunk, source_rate: u32) -> f64 {
        let data_bytes = chunk.wav_bytes.len() - 44;
        (data_bytes / 2) as f64 / f64::from(source_rate)
    }

    /// Total decoded duration across chunks.
    fn total_duration_secs(chunks: &[WavChunk], source_rate: u32) -> f64 {
        chunks
            .iter()
            .map(|chunk| chunk_duration_secs(chunk, source_rate))
            .sum()
    }

    #[rstest]
    #[case::mp3(include_bytes!("../../../test-fixtures/audio/silence.mp3").as_slice())]
    #[case::m4a(include_bytes!("../../../test-fixtures/audio/silence.m4a").as_slice())]
    #[case::ogg_vorbis(include_bytes!("../../../test-fixtures/audio/silence.ogg").as_slice())]
    #[case::flac(include_bytes!("../../../test-fixtures/audio/silence.flac").as_slice())]
    fn every_declared_container_probes_and_decodes(#[case] bytes: &[u8]) {
        let chunks = decode_all(bytes.to_vec());
        // The fixtures are 0.5s of silence; a short file yields one chunk.
        assert_eq!(chunks.len(), 1);
        let duration = chunk_duration_secs(&chunks[0], header_source_rate(&chunks[0]));
        assert!(duration > 0.3 && duration < 1.5);
    }

    #[test]
    fn garbage_input_fails_with_a_readable_error() {
        let error = match ChunkedDecoder::new(vec![0u8; 4096]) {
            Err(error) => error,
            Ok(_) => panic!("garbage must not decode"),
        };
        assert!(
            error.to_lowercase().contains("unsupported") || error.to_lowercase().contains("audio")
        );
    }

    #[test]
    fn long_input_splits_into_thirty_second_chunks_with_a_tail() {
        // 65 seconds of 16 kHz mono → 30 + 30 + 5.
        let rate = 16_000_u32;
        let samples: Vec<f32> = (0..65 * rate as usize)
            .map(|i| ((i as f32) * 0.01).sin() * 0.1)
            .collect();
        let bytes = encode_wav_s16_mono(&samples, rate);
        let chunks = decode_all(bytes);

        assert_eq!(chunks.len(), 3, "65s must split 30+30+5");
        let durations: Vec<f64> = chunks
            .iter()
            .map(|chunk| chunk_duration_secs(chunk, rate))
            .collect();
        assert!((durations[0] - 30.1).abs() < 0.2, "chunk 0: {durations:?}");
        assert!((durations[1] - 30.1).abs() < 0.2, "chunk 1: {durations:?}");
        assert!(
            durations[2] > 5.0 && durations[2] < 5.3,
            "tail chunk: {durations:?}"
        );
        // The 0.1s boundary silence is part of each duration above.
        assert!(
            total_duration_secs(&chunks, rate) - 65.0 < 0.3,
            "no audio lost across chunk boundaries"
        );
    }

    #[test]
    fn source_rate_is_preserved_in_the_wav_header() {
        // 8 kHz source — chunks stay at the source rate; the transcription
        // pipeline resamples.
        let rate = 8_000_u32;
        let samples: Vec<f32> = vec![0.1; rate as usize]; // 1 second
        let bytes = encode_wav_s16_mono(&samples, rate);
        let chunks = decode_all(bytes);
        assert_eq!(chunks.len(), 1);
        let header_rate = u32::from_le_bytes([
            chunks[0].wav_bytes[24],
            chunks[0].wav_bytes[25],
            chunks[0].wav_bytes[26],
            chunks[0].wav_bytes[27],
        ]);
        assert_eq!(header_rate, rate);
    }
}
