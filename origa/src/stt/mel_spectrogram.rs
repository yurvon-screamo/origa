use ndarray::{Array2, Array3};
use rustfft::{FftPlanner, num_complex::Complex64};

const N_FFT: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const N_FRAMES: usize = 3000;
const SAMPLE_RATE: u32 = 16_000;
const F_MIN: f64 = 0.0;
const F_MAX: f64 = 8000.0;
const N_FREQS: usize = N_FFT / 2 + 1;
const FLOOR: f64 = 1e-10;

fn hanning_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|n| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * n as f64 / size as f64).cos()))
        .collect()
}

fn hz_to_mel(hz: f64) -> f64 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f64) -> f64 {
    700.0 * (10.0_f64.powf(mel / 2595.0) - 1.0)
}

fn create_mel_filter_bank() -> Array2<f64> {
    let mel_min = hz_to_mel(F_MIN);
    let mel_max = hz_to_mel(F_MAX);

    let mel_points: Vec<f64> = (0..=N_MELS + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f64 / (N_MELS + 1) as f64)
        .collect();

    let hz_points: Vec<f64> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();

    let fft_freqs: Vec<f64> = (0..N_FREQS)
        .map(|i| i as f64 * SAMPLE_RATE as f64 / (2 * (N_FREQS - 1)) as f64)
        .collect();

    let mut bank = Array2::<f64>::zeros((N_MELS, N_FREQS));

    for m in 0..N_MELS {
        let f_left = hz_points[m];
        let f_center = hz_points[m + 1];
        let f_right = hz_points[m + 2];

        for (k, &freq) in fft_freqs.iter().enumerate() {
            let val = if freq >= f_left && freq < f_center && f_center > f_left {
                (freq - f_left) / (f_center - f_left)
            } else if freq >= f_center && freq <= f_right && f_right > f_center {
                (f_right - freq) / (f_right - f_center)
            } else {
                0.0
            };
            bank[[m, k]] = val;
        }
    }

    bank
}

fn reflect_pad(samples: &[f32]) -> Result<Vec<f64>, String> {
    if samples.is_empty() {
        return Err("Audio samples are empty".to_string());
    }

    let pad = N_FFT / 2;
    let padded_len = samples.len() + 2 * pad;
    let mut padded = vec![0.0f64; padded_len];

    let start = pad;
    for (i, &s) in samples.iter().enumerate() {
        padded[start + i] = s as f64;
    }

    for i in 0..pad {
        let src = i.min(samples.len() - 1);
        padded[pad - 1 - i] = samples[src] as f64;
    }

    for i in 0..pad {
        let src = (samples.len() - 1).saturating_sub(i);
        padded[pad + samples.len() + i] = samples[src] as f64;
    }

    Ok(padded)
}

fn compute_power_spectrum(padded: &[f64], window: &[f64]) -> Result<Array2<f64>, String> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(N_FFT);

    let mut spectrum = Array2::<f64>::zeros((N_FREQS, N_FRAMES));

    for frame in 0..N_FRAMES {
        let offset = frame * HOP_LENGTH;
        if offset + N_FFT > padded.len() {
            break;
        }

        let mut buf: Vec<Complex64> = (0..N_FFT)
            .map(|i| Complex64::new(padded[offset + i] * window[i], 0.0))
            .collect();

        fft.process(&mut buf);

        for freq in 0..N_FREQS {
            spectrum[[freq, frame]] = buf[freq].norm_sqr();
        }
    }

    Ok(spectrum)
}

pub fn compute_mel_spectrogram(samples: &[f32]) -> Result<Array3<f32>, String> {
    tracing::debug!(len = samples.len(), "Computing mel spectrogram");

    let padded = reflect_pad(samples)?;
    let window = hanning_window(N_FFT);
    let power = compute_power_spectrum(&padded, &window)?;
    let mel_fb = create_mel_filter_bank();

    let mel_spec = mel_fb.dot(&power);

    let mel_log = mel_spec.mapv(|v| v.max(FLOOR).log10());

    let frame_max: Vec<f64> = (0..N_FRAMES)
        .map(|f| {
            (0..N_MELS)
                .map(|m| mel_log[[m, f]])
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect();

    let mut mel_clamped = mel_log;
    for f in 0..N_FRAMES {
        let t = frame_max[f] - 8.0;
        for m in 0..N_MELS {
            mel_clamped[[m, f]] = mel_clamped[[m, f]].max(t);
        }
    }

    let mel_normalized = mel_clamped.mapv(|v| (v + 4.0) / 4.0);

    let output = Array3::from_shape_vec(
        (1, N_MELS, N_FRAMES),
        mel_normalized
            .as_slice()
            .expect("mel_normalized is contiguous")
            .iter()
            .map(|&v| v as f32)
            .collect(),
    )
    .expect("output shape");

    tracing::debug!(shape = ?output.shape(), "Mel spectrogram computed");
    Ok(output)
}
