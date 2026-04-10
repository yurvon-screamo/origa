#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

const TARGET_RATE: u32 = 16000;
pub const TARGET_SAMPLES: usize = 480_000; // 30s × 16000 Hz

#[cfg(not(target_arch = "wasm32"))]
pub fn load_wav(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|e| format!("Failed to open WAV: {}", e))?;
    let spec = reader.spec();

    tracing::info!(
        sample_rate = spec.sample_rate,
        channels = spec.channels,
        bits_per_sample = spec.bits_per_sample,
        "Loading WAV"
    );

    if spec.channels > 2 {
        tracing::warn!(
            channels = spec.channels,
            "Unsupported channel count, only mono and stereo are supported. Using first channels."
        );
    }

    let samples = read_samples(&mut reader, &spec)?;
    let mono = downmix_to_mono(&samples, spec.channels);
    let resampled = resample(&mono, spec.sample_rate);
    let result = pad_or_trim(&resampled, TARGET_SAMPLES);

    Ok(result)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_samples(
    reader: &mut hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: &hound::WavSpec,
) -> Result<Vec<f32>, String> {
    match spec.sample_format {
        hound::SampleFormat::Float => {
            Ok(reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect())
        },
        hound::SampleFormat::Int => {
            let max_val = match spec.bits_per_sample {
                8 => 128.0_f32,
                16 => 32768.0,
                24 => 8388608.0,
                _ => 2147483648.0,
            };
            Ok(reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max_val).unwrap_or(0.0))
                .collect())
        },
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load_audio_bytes(data: &[u8]) -> Result<Vec<f32>, String> {
    if data.len() < 44 {
        return Err("Audio data too short".to_string());
    }

    if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err("Not a WAV file".to_string());
    }

    let mut fmt_offset: Option<usize> = None;
    let mut data_offset: Option<usize> = None;
    let mut data_len: usize = 0;

    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size =
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
        if chunk_size > data.len().saturating_sub(pos + 8) {
            break;
        }

        if chunk_id == b"fmt " {
            fmt_offset = Some(pos + 8);
        } else if chunk_id == b"data" {
            data_offset = Some(pos + 8);
            data_len = chunk_size;
            break;
        }

        pos += 8 + chunk_size;
        if pos % 2 != 0 {
            pos += 1;
        }
    }

    let fmt_off = fmt_offset.ok_or("No fmt chunk found in WAV")?;
    let data_off = data_offset.ok_or("No data chunk found in WAV")?;

    if data_off >= data.len() {
        return Err("Audio data offset beyond file bounds".to_string());
    }

    if fmt_off + 16 > data.len() {
        return Err("fmt chunk too short".to_string());
    }

    let audio_format = u16::from_le_bytes([data[fmt_off], data[fmt_off + 1]]);
    let channels = u16::from_le_bytes([data[fmt_off + 2], data[fmt_off + 3]]);
    let sample_rate = u32::from_le_bytes([
        data[fmt_off + 4],
        data[fmt_off + 5],
        data[fmt_off + 6],
        data[fmt_off + 7],
    ]);
    let bits_per_sample = u16::from_le_bytes([data[fmt_off + 14], data[fmt_off + 15]]);

    tracing::info!(
        format = audio_format,
        channels = channels,
        sample_rate = sample_rate,
        bits_per_sample = bits_per_sample,
        "Parsing WAV from bytes"
    );

    let samples_end = (data_off + data_len).min(data.len());
    let sample_data = &data[data_off..samples_end];

    let samples = match audio_format {
        1 => decode_pcm(sample_data, bits_per_sample)?,
        3 => sample_data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
        _ => return Err(format!("Unsupported audio format: {}", audio_format)),
    };

    let mono = downmix_to_mono(&samples, channels);
    let resampled = resample(&mono, sample_rate);
    let result = pad_or_trim(&resampled, TARGET_SAMPLES);

    Ok(result)
}

#[cfg(target_arch = "wasm32")]
fn decode_pcm(sample_data: &[u8], bits_per_sample: u16) -> Result<Vec<f32>, String> {
    match bits_per_sample {
        8 => Ok(sample_data
            .iter()
            .map(|&b| (b as f32 - 128.0) / 128.0)
            .collect()),
        16 => {
            let max_val = 32768.0_f32;
            Ok(sample_data
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / max_val)
                .collect())
        },
        24 => {
            let max_val = 8388608.0_f32;
            Ok(sample_data
                .chunks_exact(3)
                .map(|c| {
                    let val = (c[0] as i32) | ((c[1] as i32) << 8) | ((c[2] as i32) << 16);
                    let val = if val >= 0x800000 {
                        val - 0x1000000
                    } else {
                        val
                    };
                    val as f32 / max_val
                })
                .collect())
        },
        _ => Err(format!(
            "Unsupported PCM bits_per_sample: {}",
            bits_per_sample
        )),
    }
}

pub(crate) fn downmix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        samples.to_vec()
    } else {
        samples
            .chunks(channels as usize)
            .map(|ch| ch.iter().sum::<f32>() / ch.len() as f32)
            .collect()
    }
}

pub(crate) fn resample(samples: &[f32], orig_rate: u32) -> Vec<f32> {
    if orig_rate == TARGET_RATE {
        return samples.to_vec();
    }

    let ratio = orig_rate as f64 / TARGET_RATE as f64;
    let new_len = (samples.len() as f64 / ratio).round() as usize;
    let last = samples.len() - 1;

    (0..new_len)
        .map(|i| {
            let src_idx = i as f64 * ratio;
            let idx0 = src_idx.floor() as usize;
            let idx1 = (idx0 + 1).min(last);
            let frac = src_idx - idx0 as f64;
            samples[idx0] * (1.0 - frac as f32) + samples[idx1] * frac as f32
        })
        .collect()
}

pub(crate) fn pad_or_trim(samples: &[f32], target_len: usize) -> Vec<f32> {
    if samples.len() >= target_len {
        samples[..target_len].to_vec()
    } else {
        let mut out = samples.to_vec();
        out.resize(target_len, 0.0);
        out
    }
}
