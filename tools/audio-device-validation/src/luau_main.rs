use audio_device_validation::Probe;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::time::Duration;

fn main() {
    let device = cpal::default_host()
        .default_output_device()
        .expect("default output device");
    let supported = device
        .default_output_config()
        .expect("default device format");
    assert_eq!(
        supported.sample_format(),
        cpal::SampleFormat::F32,
        "qualification adapter requires float output"
    );
    for requested in [256, 1024] {
        let mut config: cpal::StreamConfig = supported.clone().into();
        if let cpal::SupportedBufferSize::Range { min, max } = supported.buffer_size() {
            if (*min..=*max).contains(&requested) {
                config.buffer_size = cpal::BufferSize::Fixed(requested);
            }
        }
        let probe = Probe::new(config.sample_rate.0, u32::from(config.channels), requested)
            .expect("real Luau fixture");
        let output = probe.clone();
        let failed = Arc::new(AtomicBool::new(false));
        let error = failed.clone();
        let minimum = Arc::new(AtomicU32::new(u32::MAX));
        let min = minimum.clone();
        let maximum = Arc::new(AtomicU32::new(0));
        let max = maximum.clone();
        let channels = usize::from(config.channels);
        let stream = device
            .build_output_stream(
                &config,
                move |samples: &mut [f32], _| {
                    let frames = (samples.len() / channels) as u32;
                    min.fetch_min(frames, Ordering::Relaxed);
                    max.fetch_max(frames, Ordering::Relaxed);
                    output.pull(samples);
                },
                move |e| {
                    eprintln!("CPAL error: {e}");
                    error.store(true, Ordering::Release);
                },
                None,
            )
            .expect("open output");
        stream.play().expect("start output");
        for phase in 0..4 {
            probe.set_phase(phase);
            std::thread::sleep(Duration::from_millis(1200));
        }
        drop(stream);
        let stopped = probe.engine.time_in_frames();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(
            stopped,
            probe.engine.time_in_frames(),
            "device callbacks continued after stop"
        );
        let report = probe.report();
        probe.finish_controls();
        assert!(!failed.load(Ordering::Acquire));
        assert!(report.contains("\"pass\":true"), "{report}");
        println!(
            "{{\"platform\":\"macos-cpal\",\"requested_block\":{requested},\"callback_min\":{},\"callback_max\":{},\"rust\":{report}}}",
            minimum.load(Ordering::Relaxed),
            maximum.load(Ordering::Relaxed)
        );
    }
}
