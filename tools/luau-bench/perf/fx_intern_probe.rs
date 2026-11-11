use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::time::Instant;
#[derive(Default)] struct Fx(u64);
impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) { for c in bytes.chunks(8) { let mut w=[0u8;8]; for (d,s) in w.iter_mut().zip(c){*d=*s;} self.0=(self.0.rotate_left(5)^u64::from_le_bytes(w)).wrapping_mul(0x517cc1b727220a95);} }
    fn write_usize(&mut self, n: usize) { self.0=(self.0.rotate_left(5)^n as u64).wrapping_mul(0x517cc1b727220a95); }
    fn finish(&self) -> u64 { self.0 }
}
#[derive(Default)] struct FxRot(Fx);
impl Hasher for FxRot {
    fn write(&mut self, b: &[u8]) { self.0.write(b) }
    fn write_usize(&mut self, n: usize) { self.0.write_usize(n) }
    fn finish(&self) -> u64 { self.0.0.rotate_left(26) } // rustc-hash 2.x finish
}
fn run<H: Hasher + Default>(name: &str, keys: &[Vec<u8>]) {
    let mut m: HashMap<Box<[u8]>, u32, BuildHasherDefault<H>> = HashMap::default();
    let t = Instant::now();
    for (i, k) in keys.iter().enumerate() { if m.get(k.as_slice()).is_none() { m.insert(k.clone().into_boxed_slice(), i as u32); } }
    for k in keys { std::hint::black_box(m.get(k.as_slice())); }
    println!("{name:8} {:>8.1} ms for {} keys", t.elapsed().as_secs_f64()*1e3, keys.len());
}
fn main() {
    let n = 200_000;
    let sets: Vec<(&str, Vec<Vec<u8>>)> = vec![
        ("j=%d", (1..=n).map(|j| format!("j={j}").into_bytes()).collect()),
        ("j=%f", (1..=n).map(|j| format!("j={j}.000000").into_bytes()).collect()),
        ("concat", (1..=n).map(|j| format!("{j}abc").into_bytes()).collect()),
    ];
    for (name, keys) in &sets { println!("-- {name}"); run::<Fx>("fx", keys); run::<FxRot>("fx+rot", keys); }
}
