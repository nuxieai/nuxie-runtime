pub fn get_trip_count(from: f64, to: f64, step: f64) -> i32 {
    // we compute trip count in integers because that way we know that the loop math (repeated addition) is precise
    let fromi = if from >= -32767.0 && from <= 32767.0 && (from as i32 as f64) == from {
        from as i32
    } else {
        i32::MIN
    };
    let toi = if to >= -32767.0 && to <= 32767.0 && (to as i32 as f64) == to {
        to as i32
    } else {
        i32::MIN
    };
    let stepi = if step >= -32767.0 && step <= 32767.0 && (step as i32 as f64) == step {
        step as i32
    } else {
        i32::MIN
    };

    if fromi == i32::MIN || toi == i32::MIN || stepi == i32::MIN || stepi == 0 {
        return -1;
    }

    if (stepi < 0 && toi > fromi) || (stepi > 0 && toi < fromi) {
        return 0;
    }

    (toi - fromi) / stepi + 1
}
