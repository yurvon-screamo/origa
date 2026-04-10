use std::path::Path;

const TARGET_RATE: u32 = 16000;
const TARGET_SAMPLES: usize = 480_000; // 30s × 16000 Hz

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
