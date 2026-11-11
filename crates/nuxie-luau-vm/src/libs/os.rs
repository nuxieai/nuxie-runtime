//! The `os` library (C++ `loslib.cpp`): `clock`, `date`, `difftime`,
//! `time`.
//!
//! Deviation: safe Rust has no time-zone database, so local time is UTC.
//! `os.date` without `!` formats the same fields as with it (`%z` is
//! `+0000`, `%Z` is `UTC`, `isdst` is `false`), and still gives nil before
//! the epoch, as upstream does. Diff cases compare only the `!` forms and
//! fixed timestamps; the oracle uses the host's zone.
//!
//! Allocating builtins: `date` (the formatted string, checked with
//! `append` as it grows; the `*t` table, charged by `new_table`/`raw_set`).

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::sync::OnceLock;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::error::VmResult;
use crate::libs::append;
use crate::native::{Args, BuiltinFn};
use crate::value::{TableId, Value};
use crate::vm::Vm;

/// C++ `LUA_STRFTIMEOPTIONS`.
const STRFTIME_OPTIONS: &[u8] = b"aAbBcdHIjmMpSUwWxXyYzZ%";

const WEEKDAY_ABBR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const WEEKDAY_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTH_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// C `struct tm`, with C's field meanings (`mon` 0-11, `year` since 1900,
/// `wday` 0 = Sunday, `yday` 0-365).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Tm {
    sec: i64,
    min: i64,
    hour: i64,
    mday: i64,
    mon: i64,
    year: i64,
    wday: i64,
    yday: i64,
}

/// Days from 1970-01-01 to `y-m-d` (Howard Hinnant's `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y.saturating_sub(1) } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m.saturating_sub(3) } else { m.saturating_add(9) };
    let doy = mp
        .saturating_mul(153)
        .saturating_add(2)
        .wrapping_div(5)
        .saturating_add(d)
        .saturating_sub(1);
    let doe = yoe
        .saturating_mul(365)
        .saturating_add(yoe / 4)
        .saturating_sub(yoe / 100)
        .saturating_add(doy);
    era.saturating_mul(146_097)
        .saturating_add(doe)
        .saturating_sub(719_468)
}

/// C `gmtime_r`: `None` when the year does not fit C's `int`. Howard
/// Hinnant's `civil_from_days`; every intermediate is bounded by `t`.
fn gmtime(t: i64) -> Option<Tm> {
    let days = t.div_euclid(86_400);
    let rem = t.rem_euclid(86_400);
    let z = days.checked_add(719_468)?;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = doe
        .saturating_sub(doe / 1460)
        .saturating_add(doe / 36_524)
        .saturating_sub(doe / 146_096)
        / 365;
    let doy = doe.saturating_sub(
        yoe.saturating_mul(365)
            .saturating_add(yoe / 4)
            .saturating_sub(yoe / 100),
    );
    let mp = doy.saturating_mul(5).saturating_add(2) / 153;
    let mday = doy
        .saturating_sub(mp.saturating_mul(153).saturating_add(2) / 5)
        .saturating_add(1);
    let month = if mp < 10 {
        mp.saturating_add(3)
    } else {
        mp.saturating_sub(9)
    };
    let year = era
        .checked_mul(400)?
        .checked_add(yoe)?
        .checked_add(i64::from(month <= 2))?;
    let tm_year = year.checked_sub(1900)?;
    i32::try_from(tm_year).ok()?;
    let yday = days.checked_sub(days_from_civil(year, 1, 1))?;
    Some(Tm {
        sec: rem % 60,
        min: (rem % 3600) / 60,
        hour: rem / 3600,
        mday,
        mon: month.saturating_sub(1),
        year: tm_year,
        wday: days.saturating_add(4).rem_euclid(7),
        yday,
    })
}

/// C++ `os_timegm`: Julian-day arithmetic in `i64`; `None` before the
/// epoch (upstream's `-1`).
fn timegm(t: &Tm) -> Option<i64> {
    let day = t.mday;
    let month = t.mon.saturating_add(1);
    let year = t.year.saturating_add(1900);
    // C `%` and `/` truncate toward zero.
    let a = i64::from(t.mon.wrapping_rem(12) < 2).saturating_sub(t.mon.wrapping_div(12));
    let y = year.saturating_add(4800).saturating_sub(a);
    let m = month.saturating_add(a.saturating_mul(12)).saturating_sub(3);
    let julianday = day
        .saturating_add(m.saturating_mul(153).saturating_add(2).wrapping_div(5))
        .saturating_add(y.saturating_mul(365))
        .saturating_add(y.wrapping_div(4))
        .saturating_sub(y.wrapping_div(100))
        .saturating_add(y.wrapping_div(400))
        .saturating_sub(32_045);
    const EPOCH_JULIAN_DAY: i64 = 2_440_588;
    const EPOCH_JULIAN_SECOND: i64 = EPOCH_JULIAN_DAY * 86_400;
    if julianday < EPOCH_JULIAN_DAY {
        return None;
    }
    let daysecond = t
        .hour
        .saturating_mul(3600)
        .saturating_add(t.min.saturating_mul(60))
        .saturating_add(t.sec);
    let seconds = julianday.saturating_mul(86_400).saturating_add(daysecond);
    if seconds < EPOCH_JULIAN_SECOND {
        return None;
    }
    Some(seconds.saturating_sub(EPOCH_JULIAN_SECOND))
}

fn week_number(t: &Tm, monday_first: bool) -> i64 {
    let yday = t.yday.max(0);
    let wday = t.wday.rem_euclid(7);
    let wday = if monday_first {
        wday.saturating_add(6) % 7
    } else {
        wday
    };
    yday.saturating_add(7).saturating_sub(wday) / 7
}

fn name(names: &[&'static str], i: i64) -> &'static str {
    usize::try_from(i)
        .ok()
        .and_then(|i| names.get(i))
        .copied()
        .unwrap_or("")
}

/// One `strftime` conversion in the C locale (the vendored
/// `strftime_directive`).
fn directive(t: &Tm, conv: u8) -> String {
    let wday = t.wday.rem_euclid(7);
    let mon = t.mon.rem_euclid(12);
    let year = t.year.saturating_add(1900);
    let hour = t.hour.rem_euclid(24);
    match conv {
        b'a' => name(&WEEKDAY_ABBR, wday).to_owned(),
        b'A' => name(&WEEKDAY_FULL, wday).to_owned(),
        b'b' => name(&MONTH_ABBR, mon).to_owned(),
        b'B' => name(&MONTH_FULL, mon).to_owned(),
        b'c' => format!(
            "{} {} {:2} {:02}:{:02}:{:02} {}",
            name(&WEEKDAY_ABBR, wday),
            name(&MONTH_ABBR, mon),
            t.mday,
            t.hour,
            t.min,
            t.sec,
            year
        ),
        b'd' => format!("{:02}", t.mday),
        b'H' => format!("{:02}", t.hour),
        b'I' => format!("{:02}", if hour % 12 == 0 { 12 } else { hour % 12 }),
        b'j' => format!("{:03}", t.yday.saturating_add(1)),
        b'm' => format!("{:02}", t.mon.saturating_add(1)),
        b'M' => format!("{:02}", t.min),
        b'p' => (if hour < 12 { "AM" } else { "PM" }).to_owned(),
        b'S' => format!("{:02}", t.sec),
        b'U' => format!("{:02}", week_number(t, false)),
        b'w' => format!("{wday}"),
        b'W' => format!("{:02}", week_number(t, true)),
        b'x' => format!(
            "{:02}/{:02}/{:02}",
            t.mon.saturating_add(1),
            t.mday,
            year.rem_euclid(100)
        ),
        b'X' => format!("{:02}:{:02}:{:02}", t.hour, t.min, t.sec),
        b'y' => format!("{:02}", year.rem_euclid(100)),
        b'Y' => format!("{year}"),
        b'z' => "+0000".to_owned(),
        b'Z' => "UTC".to_owned(),
        b'%' => "%".to_owned(),
        _ => String::new(),
    }
}

/// C `time(NULL)`: whole seconds since the epoch.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(e) => i64::try_from(e.duration().as_secs()).map_or(i64::MIN, |s| s.saturating_neg()),
    }
}

/// `wasm32-unknown-unknown` has no clock (`SystemTime::now` panics there).
/// The vendored VM's wasm shim (`luaur-common` `wasm_libc::time`) reports
/// a fixed instant, 2024-01-01T00:00:00Z; so does this.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn now() -> i64 {
    1_704_067_200
}

/// C++ `lua_clock`: a monotonic timestamp in seconds from an arbitrary
/// origin (here, the first call in the process).
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn clock_seconds() -> f64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64()
}

/// No clock on `wasm32-unknown-unknown` (`Instant::now` panics there); the
/// vendored wasm shim's `clock` reports 0, and so does this.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn clock_seconds() -> f64 {
    0.0
}

fn clock(vm: &mut Vm, _args: Args) -> VmResult<usize> {
    vm.ret(&[Value::Number(clock_seconds())])
}

fn difftime(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t1 = vm.check_number(args, 1)?;
    let t2 = if vm.arg_or_nil(args, 2).is_nil() {
        0.0
    } else {
        vm.check_number(args, 2)?
    };
    vm.ret(&[Value::Number(t1 - t2)])
}

/// C++ `getfield`: an integer field of the date table, or `default`
/// (an error when `default` is negative).
fn getfield(vm: &mut Vm, t: TableId, key: &str, default: i32) -> VmResult<i64> {
    let k = vm.heap.intern(key.as_bytes())?;
    let v = vm.heap.table(t).get(Value::String(k));
    match vm.to_number(v) {
        // `lua_tointeger`: `luai_num2int`, a cast to `int`.
        Some(x) => Ok(i64::from(x as i32)),
        None if default < 0 => {
            Err(vm.lib_error(format!("field '{key}' missing in date table")))
        }
        None => Ok(i64::from(default)),
    }
}

fn time(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = if vm.arg_or_nil(args, 1).is_nil() {
        Some(now())
    } else {
        let table = vm.check_table(args, 1)?;
        // C `int` fields: `month - 1` and `year - 1900` wrap as the
        // vendored code's `wrapping_sub` does.
        let sec = getfield(vm, table, "sec", 0)?;
        let min = getfield(vm, table, "min", 0)?;
        let hour = getfield(vm, table, "hour", 12)?;
        let mday = getfield(vm, table, "day", -1)?;
        let mon = (getfield(vm, table, "month", -1)? as i32).wrapping_sub(1);
        let year = (getfield(vm, table, "year", -1)? as i32).wrapping_sub(1900);
        // `isdst` is read but does not change `os_timegm`'s result.
        let tm = Tm {
            sec,
            min,
            hour,
            mday,
            mon: i64::from(mon),
            year: i64::from(year),
            ..Tm::default()
        };
        timegm(&tm)
    };
    match t {
        Some(t) => vm.ret(&[Value::Number(t as f64)]),
        None => vm.ret(&[Value::Nil]),
    }
}

fn date_table(vm: &mut Vm, tm: &Tm) -> VmResult<usize> {
    let t = vm.new_table(0, 9)?;
    let fields = [
        ("sec", tm.sec),
        ("min", tm.min),
        ("hour", tm.hour),
        ("day", tm.mday),
        ("month", tm.mon.saturating_add(1)),
        ("year", tm.year.saturating_add(1900)),
        ("wday", tm.wday.saturating_add(1)),
        ("yday", tm.yday.saturating_add(1)),
    ];
    for (key, value) in fields {
        // `setfield` pushes through C `int`.
        vm.raw_set_str(t, key, Value::Number(f64::from(value as i32)))?;
    }
    vm.raw_set_str(t, "isdst", Value::Boolean(false))?;
    vm.ret(&[Value::Table(t)])
}

fn date(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let format = match vm.opt_string(args, 1)? {
        Some(s) => vm.heap.str_rc(s),
        None => std::rc::Rc::from(&b"%c"[..]),
    };
    let t = if vm.arg_or_nil(args, 2).is_nil() {
        now()
    } else {
        // C `(time_t)` of a double; saturating here.
        vm.check_number(args, 2)? as i64
    };
    // C reads the format through its NUL terminator.
    let end = format.iter().position(|&b| b == 0).unwrap_or(format.len());
    let mut s = format.get(..end).unwrap_or_default();
    let tm = if let Some(rest) = s.strip_prefix(b"!") {
        s = rest;
        gmtime(t)
    } else if t < 0 {
        // Upstream refuses local times before the epoch.
        None
    } else {
        gmtime(t)
    };
    let Some(tm) = tm else {
        return vm.ret(&[Value::Nil]);
    };
    if s == b"*t" {
        return date_table(vm, &tm);
    }
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(&c) = s.get(i) {
        match s.get(i.saturating_add(1)) {
            Some(&conv) if c == b'%' => {
                if !STRFTIME_OPTIONS.contains(&conv) {
                    return Err(vm.arg_error(1, "invalid conversion specifier"));
                }
                append(vm, &mut out, directive(&tm, conv).as_bytes())?;
                i = i.saturating_add(2);
            }
            _ => {
                append(vm, &mut out, &[c])?;
                i = i.saturating_add(1);
            }
        }
    }
    vm.ret_str(&out)
}

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let functions: &[(&str, BuiltinFn)] = &[
        ("clock", clock),
        ("date", date),
        ("difftime", difftime),
        ("time", time),
    ];
    vm.open_lib("os", functions)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmtime_round_trips_through_timegm() {
        for t in [0, 1, 59, 86_399, 951_782_400, 960_595_200, 32_535_215_999] {
            let tm = gmtime(t);
            assert_eq!(tm.and_then(|tm| timegm(&tm)), Some(t));
        }
    }

    #[test]
    fn gmtime_fields() {
        assert_eq!(
            gmtime(-1),
            Some(Tm {
                sec: 59,
                min: 59,
                hour: 23,
                mday: 31,
                mon: 11,
                year: 69,
                wday: 3,
                yday: 364,
            })
        );
        assert_eq!(gmtime(i64::MAX), None);
    }
}
