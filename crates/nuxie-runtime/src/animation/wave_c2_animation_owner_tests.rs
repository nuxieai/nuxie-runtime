use super::*;

fn animation(speed: f32, loop_value: u64, work_area: bool) -> RuntimeLinearAnimation {
    RuntimeLinearAnimation {
        global_id: 2,
        name: Some(Arc::<str>::from("upstream unit-test animation")),
        fps: 2,
        duration: if work_area { 100 } else { 10 },
        speed,
        loop_value,
        work_start: if work_area { 4 } else { 0 },
        work_end: if work_area { 10 } else { 0 },
        enable_work_area: work_area,
        quantize: false,
        keyed_objects: Arc::new(Vec::new()),
        key_frame_data_bind_templates: Arc::new(Vec::new()),
        has_keyed_callbacks: false,
    }
}

fn instance(animation: &RuntimeLinearAnimation) -> LinearAnimationInstance {
    LinearAnimationInstance::new_for_test(RuntimeLinearAnimationHandle::new(0), animation, 1.0)
}

fn end_time(definition: &RuntimeLinearAnimation, speed_multiplier: f32) -> f32 {
    if definition.speed * speed_multiplier < 0.0 {
        definition.start_seconds()
    } else {
        definition.end_seconds()
    }
}

#[test]
fn wave_c2_linear_instance_001_one_shot() {
    let definition = animation(1.0, 0, false);
    let mut occurrence = instance(&definition);
    assert!(occurrence.advance(2.0));
    assert_eq!(occurrence.time(), 2.0);
    assert_eq!(occurrence.total_time(), 2.0);
    assert!(!occurrence.did_loop());
    assert!(!occurrence.advance(10.0));
    assert_eq!(occurrence.time(), 5.0);
    assert_eq!(occurrence.total_time(), 12.0);
    assert!(occurrence.did_loop());
}

#[test]
fn wave_c2_linear_instance_002_speed() {
    let definition = animation(0.5, 0, false);
    let mut occurrence = instance(&definition);
    assert!(occurrence.advance(2.0));
    assert_eq!(occurrence.time(), 1.0);
    assert_eq!(occurrence.total_time(), 1.0);
}

#[test]
fn wave_c2_linear_instance_003_negative_advance_adds_absolute_total_time() {
    let definition = animation(1.0, 1, false);
    let mut occurrence = instance(&definition);
    assert!(occurrence.advance(-2.0));
    assert_eq!(occurrence.time(), 3.0);
    assert_eq!(occurrence.total_time(), 2.0);
    assert!(occurrence.did_loop());
}

#[test]
fn wave_c2_linear_instance_004_reverse_one_shot() {
    let definition = animation(1.0, 0, false);
    let mut occurrence = instance(&definition);
    occurrence.set_direction(-1);
    assert_eq!(occurrence.time(), 0.0);
    assert!(!occurrence.advance(2.0));
    assert_eq!(occurrence.time(), 0.0);
    assert_eq!(occurrence.total_time(), 2.0);
    assert!(occurrence.did_loop());
    occurrence.set_time(&definition, 5.0);
    assert_eq!(occurrence.total_time(), 5.0);
    occurrence.set_direction(-1);
    assert!(occurrence.advance(2.0));
    assert_eq!(occurrence.time(), 3.0);
    assert_eq!(occurrence.total_time(), 7.0);
    assert!(!occurrence.did_loop());
    assert!(!occurrence.advance(4.0));
    assert_eq!(occurrence.time(), 0.0);
    assert_eq!(occurrence.total_time(), 11.0);
    assert!(occurrence.did_loop());
}

#[test]
fn wave_c2_linear_instance_005_forward_loop() {
    let definition = animation(1.0, 1, false);
    let mut occurrence = instance(&definition);
    assert!(occurrence.advance(2.0));
    assert_eq!(occurrence.time(), 2.0);
    assert_eq!(occurrence.total_time(), 2.0);
    assert!(!occurrence.did_loop());
    assert!(occurrence.advance(10.0));
    assert_eq!(occurrence.time(), 2.0);
    assert_eq!(occurrence.total_time(), 12.0);
    assert!(occurrence.did_loop());
}

#[test]
fn wave_c2_linear_instance_006_reverse_loop() {
    let definition = animation(1.0, 1, false);
    let mut occurrence = instance(&definition);
    occurrence.set_direction(-1);
    assert_eq!(occurrence.time(), 0.0);
    for (elapsed, time, total, looped) in [
        (2.0, 3.0, 2.0, true),
        (2.0, 1.0, 4.0, false),
        (4.0, 2.0, 8.0, true),
    ] {
        assert!(occurrence.advance(elapsed));
        assert_eq!(occurrence.direction(), -1.0);
        assert_eq!(occurrence.time(), time);
        assert_eq!(occurrence.total_time(), total);
        assert_eq!(occurrence.did_loop(), looped);
    }
}

#[test]
fn wave_c2_linear_instance_007_reverse_loop_work_area() {
    let definition = animation(1.0, 1, true);
    let mut occurrence = instance(&definition);
    occurrence.set_direction(-1);
    assert_eq!(occurrence.time(), 2.0);
    assert!(!occurrence.advance(0.0));
    assert_eq!(occurrence.direction(), -1.0);
    assert_eq!(occurrence.time(), 2.0);
    assert_eq!(occurrence.total_time(), 0.0);
    assert!(!occurrence.did_loop());
    for (time, total) in [(3.0, 2.0), (4.0, 4.0), (5.0, 6.0)] {
        assert!(occurrence.advance(2.0));
        assert_eq!(occurrence.direction(), -1.0);
        assert_eq!(occurrence.time(), time);
        assert_eq!(occurrence.total_time(), total);
        assert!(occurrence.did_loop());
    }
}

#[test]
fn wave_c2_linear_instance_008_forward_ping_pong() {
    let definition = animation(1.0, 2, false);
    let mut occurrence = instance(&definition);
    for (elapsed, time, total, direction, looped) in [
        (2.0, 2.0, 2.0, 1.0, false),
        (5.0, 3.0, 7.0, -1.0, true),
        (9.0, 4.0, 16.0, -1.0, true),
        (6.0, 2.0, 22.0, 1.0, true),
        (20.0, 2.0, 42.0, 1.0, true),
    ] {
        assert!(occurrence.advance(elapsed));
        assert_eq!(occurrence.time(), time);
        assert_eq!(occurrence.total_time(), total);
        assert_eq!(occurrence.direction(), direction);
        assert_eq!(occurrence.did_loop(), looped);
    }
}

#[test]
fn wave_c2_linear_instance_009_reverse_ping_pong() {
    let definition = animation(1.0, 2, false);
    let mut occurrence = instance(&definition);
    occurrence.set_direction(-1);
    assert_eq!(occurrence.time(), 0.0);
    for (elapsed, time, total, direction, looped) in [
        (2.0, 2.0, 2.0, 1.0, true),
        (4.0, 4.0, 6.0, -1.0, true),
        (2.0, 2.0, 8.0, -1.0, false),
    ] {
        assert!(occurrence.advance(elapsed));
        assert_eq!(occurrence.time(), time);
        assert_eq!(occurrence.total_time(), total);
        assert_eq!(occurrence.direction(), direction);
        assert_eq!(occurrence.did_loop(), looped);
    }
}

#[test]
fn wave_c2_linear_instance_010_override_loop() {
    let definition = animation(1.0, 0, false);
    let mut occurrence = instance(&definition);
    assert_eq!(occurrence.loop_value(), definition.loop_value as i32);
    occurrence.set_loop_value(2);
    assert_ne!(occurrence.loop_value(), definition.loop_value as i32);
    assert_eq!(occurrence.loop_value(), 2);
    assert_eq!(
        occurrence.resolved_loop_kind(&definition),
        AnimationLoop::PingPong
    );
}

#[test]
fn wave_c2_linear_definition_001_positive_speed_times() {
    let definition = animation(1.0, 0, false);
    assert_eq!(definition.start_seconds(), 0.0);
    assert_eq!(definition.end_seconds(), 5.0);
    assert_eq!(definition.start_time_with_speed(1.0), 0.0);
    assert_eq!(end_time(&definition, 1.0), 5.0);
    assert_eq!(definition.duration_seconds(), 5.0);
}

#[test]
fn wave_c2_linear_definition_002_negative_speed_times() {
    let definition = animation(-1.0, 0, false);
    assert_eq!(definition.start_seconds(), 0.0);
    assert_eq!(definition.end_seconds(), 5.0);
    assert_eq!(definition.start_time_with_speed(1.0), 5.0);
    assert_eq!(end_time(&definition, 1.0), 0.0);
    assert_eq!(definition.duration_seconds(), 5.0);
}

#[test]
fn wave_c2_linear_definition_004_keep_going_work_area() {
    let definition = RuntimeLinearAnimation {
        global_id: 3,
        name: None,
        fps: 60,
        duration: 60,
        speed: 1.0,
        loop_value: 0,
        work_start: 30,
        work_end: 42,
        enable_work_area: true,
        quantize: false,
        keyed_objects: Arc::new(Vec::new()),
        key_frame_data_bind_templates: Arc::new(Vec::new()),
        has_keyed_callbacks: false,
    };
    let mut occurrence = instance(&definition);
    assert!(!occurrence.advance(0.0));
    assert_eq!(occurrence.time(), 0.5);
    assert!(occurrence.advance(0.1));
    assert_eq!(occurrence.time(), 0.6);
    assert!(!occurrence.advance(0.2));
    assert_eq!(occurrence.time(), 0.7);
}
