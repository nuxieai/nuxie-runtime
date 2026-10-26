use audio_device_validation::Probe;
use nuxie_audio::{AudioDeviceSink, AudioEngine};
use std::sync::atomic::Ordering;
use std::time::Duration;

fn main() {
    for requested in [256, 1024] {
        AudioEngine::set_buffer_frames(requested);
        let sink = AudioDeviceSink::open_default().expect("open actual default audio device");
        AudioEngine::set_buffer_frames(0);
        let probe = Probe::from_engine(sink.engine().clone());
        let mut records = Vec::new();
        let mut peaks = Vec::new();
        for phase in 0..4 {
            probe.set_phase(phase);
            // Drain bounded read-ahead before measuring steady output levels.
            std::thread::sleep(Duration::from_millis(150));
            let mut levels = vec![0.0; probe.engine.channels() as usize];
            probe.engine.levels(&mut levels);
            let start = probe.engine.time_in_frames();
            std::thread::sleep(Duration::from_millis(750));
            probe.engine.levels(&mut levels);
            let end = probe.engine.time_in_frames();
            let peak = levels.iter().copied().fold(0.0_f32, f32::max);
            assert!(!sink.output_failed(), "CPAL asynchronous output failure");
            assert!(end > start, "device did not drain the mixer");
            assert!(peak.is_finite() && peak < 0.1);
            if phase < 3 {
                assert!(peak > 0.005, "missing mixed output");
            } else {
                assert_eq!(peak, 0.0, "output did not silence after stop");
            }
            peaks.push(peak);
            records.push(format!(
                "{{\"phase\":{phase},\"start_frame\":{start},\"end_frame\":{end},\"peak\":{peak}}}"
            ));
        }
        assert!(
            peaks[0] > peaks[1] && peaks[1] > peaks[2],
            "parameter change must alter actual mixed levels: {peaks:?}"
        );
        let calls = probe
            .signals
            .calls
            .each_ref()
            .map(|v| v.load(Ordering::Acquire));
        assert!(calls[0] > 0 && calls[1] > 0);
        assert_eq!(&calls[2..], &[0, 0]);
        assert_eq!(probe.signals.drops.load(Ordering::Acquire), 1);
        assert_eq!(probe.signals.clock_errors.load(Ordering::Acquire), 0);
        drop(sink);
        let stopped = probe.engine.time_in_frames();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(
            probe.engine.time_in_frames(),
            stopped,
            "callback continued after device teardown"
        );
        println!(
            "{{\"platform\":\"macos-cpal\",\"pass\":true,\"requested_block\":{requested},\"actual_block\":{},\"rate\":{},\"channels\":{},\"renderer_calls\":{:?},\"renderer_drops\":1,\"teardown_clock\":{stopped},\"phases\":[{}]}}",
            probe.engine.buffer_frames(),
            probe.engine.sample_rate(),
            probe.engine.channels(),
            calls,
            records.join(",")
        );
    }
}
