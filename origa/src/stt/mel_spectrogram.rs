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

#[cfg(test)]
mod tests {
    use super::{
        N_FFT, N_FRAMES, N_MELS, compute_mel_spectrogram, create_mel_filter_bank, hanning_window,
        hz_to_mel, mel_to_hz, reflect_pad,
    };
    use std::f64::consts::PI;

    /// Analytic golden for digital silence: zero power spectrum →
    /// log10(FLOOR) = −10, the frame-max clamp (−18) does not bite, and the
    /// normalization gives (−10 + 4) / 4 = −1.5 exactly.
    const SILENCE_VALUE: f64 = -1.5;

    /// A deterministic 440 Hz sine at 16 kHz — the Whisper-native rate, so
    /// the frequency axis is exercised without resampling noise.
    fn sine_440hz(seconds: f64) -> Vec<f32> {
        let len = (seconds * 16_000.0) as usize;
        (0..len)
            .map(|i| (2.0 * PI * 440.0 * i as f64 / 16_000.0).sin() as f32 * 0.5)
            .collect()
    }

    #[test]
    fn hz_mel_conversions_roundtrip() {
        for hz in [0.0, 100.0, 440.0, 1000.0, 4000.0, 8000.0] {
            let roundtripped = mel_to_hz(hz_to_mel(hz));
            assert!(
                (roundtripped - hz).abs() < 1e-6,
                "{hz} Hz roundtripped to {roundtripped}"
            );
        }
    }

    #[test]
    fn hanning_window_is_periodic_symmetric_and_peaks_in_the_middle() {
        let window = hanning_window(N_FFT);
        assert_eq!(window.len(), N_FFT);
        assert!(window[0].abs() < 1e-9);
        let mid = N_FFT / 2;
        assert!((window[mid] - 1.0).abs() < 1e-9);
        // Periodic (DFT-even) window: w[n] == w[size - n].
        for n in 1..mid {
            assert!(
                (window[n] - window[N_FFT - n]).abs() < 1e-12,
                "window not periodic-symmetric at {n}"
            );
        }
    }

    #[test]
    fn mel_filter_bank_rows_form_triangles_within_unit_range() {
        let bank = create_mel_filter_bank();
        assert_eq!(bank.shape(), &[N_MELS, N_FFT / 2 + 1]);
        for m in 0..N_MELS {
            let row = bank.row(m);
            let mut peak = 0.0f64;
            for &v in row.iter() {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "filter {m} value {v} out of range"
                );
                peak = peak.max(v);
            }
            assert!(peak > 0.0, "filter {m} never overlaps the spectrum");
        }
    }

    #[test]
    fn reflect_pad_mirrors_edges_and_rejects_empty_input() {
        assert!(reflect_pad(&[]).is_err());
        let padded = reflect_pad(&[1.0, 2.0, 3.0]).expect("pad");
        assert_eq!(padded.len(), 3 + N_FFT);
        let pad = N_FFT / 2;
        assert_eq!(padded[pad], 1.0, "body starts with the first sample");
        assert_eq!(padded[pad - 1], 1.0, "left edge mirrors the first sample");
        assert_eq!(padded[pad + 3], 3.0, "right edge mirrors the last sample");
    }

    #[test]
    fn silence_maps_every_element_to_the_floor_value() {
        // Analytic golden: zero spectrum → log10(FLOOR) = -10 → frame max -10,
        // clamp t = -18 does not bite, normalization gives (−10+4)/4 = −1.5.
        let result = compute_mel_spectrogram(&[0.0; 8000]).expect("silence");
        assert_eq!(result.shape(), &[1, N_MELS, N_FRAMES]);
        for &v in result.iter() {
            assert!((v as f64 - SILENCE_VALUE).abs() < 1e-6);
        }
    }

    #[test]
    fn short_input_leaves_trailing_frames_at_the_floor_value() {
        // Arrange — 1/8 s of tone: frames past the signal end have no energy.
        let result = compute_mel_spectrogram(&sine_440hz(0.125)).expect("short tone");

        // Assert — early frames carry energy, tail is exactly silence.
        let early = result[[0, 20, 5]] as f64;
        assert!(early > SILENCE_VALUE, "toned frame must exceed the floor");
        let tail = result[[0, 20, N_FRAMES - 10]] as f64;
        assert!((tail - SILENCE_VALUE).abs() < 1e-6);
    }

    #[test]
    fn sine_tone_peaks_near_the_mel_band_of_its_frequency() {
        // Arrange — 2 s of 440 Hz. mel(440) ≈ 549 sits between band centers
        // 14 (≈416 Hz) and 15 (≈451 Hz); averaging over voiced frames must
        // place the peak in the 14–16 window. Catches a swapped hz↔mel axis
        // or an off-by-one in the band layout.
        let result = compute_mel_spectrogram(&sine_440hz(2.0)).expect("sine");
        let voiced_frames = 200;

        // Assert — shape, finiteness, energy above floor in voiced frames.
        assert_eq!(result.shape(), &[1, N_MELS, N_FRAMES]);
        let mut band_energy = vec![0.0f64; N_MELS];
        for m in 0..N_MELS {
            for f in 0..voiced_frames {
                let v = result[[0, m, f]] as f64;
                assert!(v.is_finite(), "non-finite value at band {m} frame {f}");
                band_energy[m] += v;
            }
        }
        let peak_band = band_energy
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .expect("non-empty bands")
            .0;
        assert!(
            (14..=16).contains(&peak_band),
            "440 Hz peaked in band {peak_band}, expected 14–16"
        );
    }

    #[test]
    fn computation_is_deterministic() {
        // Arrange
        let input = sine_440hz(0.25);

        // Act
        let first = compute_mel_spectrogram(&input).expect("first run");
        let second = compute_mel_spectrogram(&input).expect("second run");

        // Assert
        assert_eq!(first.as_slice(), second.as_slice());
    }
}
