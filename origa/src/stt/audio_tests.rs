use super::audio::*;

#[test]
fn downmix_to_mono_stereo() {
    let stereo = vec![1.0, 0.5, -1.0, -0.5, 0.0, 1.0];
    let mono = downmix_to_mono(&stereo, 2);
    assert_eq!(mono.len(), 3);
    assert!((mono[0] - 0.75).abs() < 0.001);
    assert!((mono[1] - (-0.75)).abs() < 0.001);
    assert!((mono[2] - 0.5).abs() < 0.001);
}

#[test]
fn downmix_to_mono_already_mono() {
    let mono = vec![1.0, 2.0, 3.0];
    let result = downmix_to_mono(&mono, 1);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn resample_same_rate() {
    let samples = vec![1.0, 2.0, 3.0, 4.0];
    let result = resample(&samples, 16000);
    assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn resample_downsample() {
    let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let result = resample(&samples, 48000);
    assert_eq!(result.len(), 2);
    assert!((result[0] - 0.0).abs() < 0.001);
    assert!((result[1] - 3.0).abs() < 0.001);
}

#[test]
fn resample_upsample() {
    let samples = vec![0.0, 2.0, 4.0];
    let result = resample(&samples, 8000);
    assert_eq!(result.len(), 6);
    assert!((result[0] - 0.0).abs() < 0.001);
    assert!((result[1] - 1.0).abs() < 0.001);
    assert!((result[2] - 2.0).abs() < 0.001);
}

#[test]
fn pad_or_trim_trim() {
    let samples = vec![1.0; 100];
    let result = pad_or_trim(&samples, 50);
    assert_eq!(result.len(), 50);
}

#[test]
fn pad_or_trim_pad() {
    let samples = vec![1.0, 2.0, 3.0];
    let result = pad_or_trim(&samples, 10);
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[2], 3.0);
    assert_eq!(result[3], 0.0);
    assert_eq!(result[9], 0.0);
}

#[test]
fn pad_or_trim_exact() {
    let samples = vec![1.0, 2.0, 3.0];
    let result = pad_or_trim(&samples, 3);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

// ---------------------------------------------------------------------------
// load_wav — native WAV file pipeline (hound): read → downmix → resample → pad
// ---------------------------------------------------------------------------

fn write_wav_file(path: &std::path::Path, spec: hound::WavSpec, samples: &[i16]) {
    let mut writer = hound::WavWriter::create(path, spec).expect("wav writer");
    for &s in samples {
        writer.write_sample(s).expect("write sample");
    }
    writer.finalize().expect("finalize");
}

#[test]
fn load_wav_pads_mono_int16_to_thirty_seconds() {
    // Arrange
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("mono.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    write_wav_file(&path, spec, &[100, -100, 500, -500]);

    // Act
    let samples = load_wav(&path).expect("load_wav");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 100.0 / 32768.0).abs() < 1e-6);
    assert!((samples[1] - (-100.0) / 32768.0).abs() < 1e-6);
    assert_eq!(samples[4], 0.0, "padding beyond the input is silence");
}

#[test]
fn load_wav_downmixes_stereo_and_resamples() {
    // Arrange — 48 kHz stereo: L=2000, R=1000 → mono 1500; two frames only.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("stereo.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    write_wav_file(&path, spec, &[2000, 1000, -2000, -1000]);

    // Act
    let samples = load_wav(&path).expect("load_wav");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    // First resampled frame lands at source index 0 → mean(2000, 1000)/32768.
    assert!((samples[0] - 1500.0 / 32768.0).abs() < 1e-6);
}

#[test]
fn load_wav_trims_longer_than_thirty_seconds() {
    // Arrange — 48001 samples at 16 kHz exceeds the 30 s window by one.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("long.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let raw: Vec<i16> = (0..48001).map(|i| (i % 7) as i16).collect();
    write_wav_file(&path, spec, &raw);

    // Act
    let samples = load_wav(&path).expect("load_wav");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 0.0).abs() < 1e-9);
}

// ---------------------------------------------------------------------------
// load_audio_bytes — in-memory RIFF/WAVE walk (byte-level contract)
// ---------------------------------------------------------------------------

/// Minimal RIFF/WAVE payload: optional extra chunks, fmt (PCM or float),
/// data with the given samples.
fn wav_bytes(
    channels: u16,
    sample_rate: u32,
    audio_format: u16,
    bits_per_sample: u16,
    data: &[u8],
    extra_chunk_before_data: bool,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    let fmt_size = 16u32;
    let riff_size =
        4 + 8 + fmt_size + 8 + data.len() as u32 + if extra_chunk_before_data { 8 + 4 } else { 0 };
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");

    if extra_chunk_before_data {
        // LIST chunk with an odd payload (4 bytes) exercising the padding walk.
        bytes.extend_from_slice(b"LIST");
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"INFO");
    }

    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&fmt_size.to_le_bytes());
    bytes.extend_from_slice(&audio_format.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample) / 8;
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * bits_per_sample / 8;
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());

    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}

#[test]
fn load_audio_bytes_decodes_pcm16_mono() {
    // Arrange
    let mut data = Vec::new();
    data.extend_from_slice(&3i16.to_le_bytes());
    data.extend_from_slice(&(-3i16).to_le_bytes());
    let bytes = wav_bytes(1, 16_000, 1, 16, &data, false);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 3.0 / 32768.0).abs() < 1e-6);
    assert!((samples[1] - (-3.0) / 32768.0).abs() < 1e-6);
    assert_eq!(samples[2], 0.0);
}

#[test]
fn load_audio_bytes_decodes_pcm8() {
    // Arrange — PCM8 is unsigned with a 128 offset: 128 → silence, 255 → +1.
    let bytes = wav_bytes(1, 16_000, 1, 8, &[128, 255, 0], false);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 0.0).abs() < 1e-6);
    assert!((samples[1] - 127.0 / 128.0).abs() < 1e-6);
    assert!((samples[2] - (-1.0)).abs() < 1e-6);
}

#[test]
fn load_audio_bytes_decodes_pcm24_with_sign_extension() {
    // Arrange — -1 as 24-bit LE: FF FF FF; +1 as 24-bit: 01 00 00.
    let bytes = wav_bytes(
        1,
        16_000,
        1,
        24,
        &[0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00],
        false,
    );

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - (-1.0) / 8388608.0).abs() < 1e-12);
    assert!((samples[1] - 1.0 / 8388608.0).abs() < 1e-12);
}

#[test]
fn load_audio_bytes_decodes_float32() {
    // Arrange
    let mut data = Vec::new();
    data.extend_from_slice(&0.25f32.to_le_bytes());
    data.extend_from_slice(&(-0.75f32).to_le_bytes());
    let bytes = wav_bytes(1, 16_000, 3, 32, &data, false);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 0.25).abs() < 1e-6);
    assert!((samples[1] - (-0.75)).abs() < 1e-6);
}

#[test]
fn load_audio_bytes_downmixes_stereo_before_resampling() {
    // Arrange — one stereo frame L=2000, R=1000 at 16 kHz (no resampling).
    let mut data = Vec::new();
    data.extend_from_slice(&2000i16.to_le_bytes());
    data.extend_from_slice(&1000i16.to_le_bytes());
    let bytes = wav_bytes(2, 16_000, 1, 16, &data, false);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 1500.0 / 32768.0).abs() < 1e-6);
}

#[test]
fn load_audio_bytes_walks_extra_chunks_and_odd_padding() {
    // Arrange — LIST chunk with odd payload before data.
    let mut data = Vec::new();
    data.extend_from_slice(&100i16.to_le_bytes());
    let bytes = wav_bytes(1, 16_000, 1, 16, &data, true);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert — data was still found and decoded.
    assert_eq!(samples.len(), TARGET_SAMPLES);
    assert!((samples[0] - 100.0 / 32768.0).abs() < 1e-6);
}

#[test]
fn load_audio_bytes_resamples_to_sixteen_khz() {
    // Arrange — 48 kHz mono, 48 samples (1 ms).
    let data: Vec<u8> = (0..48)
        .flat_map(|i| {
            let s = if i % 2 == 0 { 100i16 } else { -100i16 };
            s.to_le_bytes()
        })
        .collect();
    let bytes = wav_bytes(1, 48_000, 1, 16, &data, false);

    // Act
    let samples = load_audio_bytes(&bytes).expect("decode");

    // Assert — length is padded to 30 s regardless of the source rate.
    assert_eq!(samples.len(), TARGET_SAMPLES);
}

#[test]
fn load_audio_bytes_rejects_payload_shorter_than_header() {
    let result = load_audio_bytes(b"short");
    assert_eq!(result.unwrap_err(), "Audio data too short");
}

#[test]
fn load_audio_bytes_rejects_non_riff_payload() {
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"OGG!");
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "Not a WAV file");
}

#[test]
fn load_audio_bytes_rejects_missing_fmt_chunk() {
    // Arrange — valid RIFF header, data chunk only (padded to ≥44 bytes).
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(&[0u8; 20]); // filler to pass the length guard
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "No fmt chunk found in WAV");
}

#[test]
fn load_audio_bytes_rejects_missing_data_chunk() {
    // Arrange — LIST + fmt chunks only, no data (≥44 bytes total).
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"LIST");
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(b"INFO");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&16_000u32.to_le_bytes());
    bytes.extend_from_slice(&32_000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "No data chunk found in WAV");
}

#[test]
fn load_audio_bytes_rejects_oversized_data_chunk_declaration() {
    // Arrange — data chunk declares more bytes than the payload carries:
    // the walk breaks out and reports the missing data chunk.
    let mut bytes = wav_bytes(1, 16_000, 1, 16, &[0, 0], false);
    // data length field sits right before the payload: find it via the walk layout
    // (44-byte standard header layout used by wav_bytes without extra chunks).
    let data_len_pos = 40;
    bytes[data_len_pos..data_len_pos + 4].copy_from_slice(&9999u32.to_le_bytes());
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "No data chunk found in WAV");
}

#[test]
fn load_audio_bytes_rejects_unsupported_audio_format() {
    let bytes = wav_bytes(1, 16_000, 7, 16, &[0, 0], false);
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "Unsupported audio format: 7");
}

#[test]
fn load_audio_bytes_rejects_unsupported_pcm_bit_depth() {
    let bytes = wav_bytes(1, 16_000, 1, 12, &[0, 0], false);
    let result = load_audio_bytes(&bytes);
    assert_eq!(result.unwrap_err(), "Unsupported PCM bits_per_sample: 12");
}
