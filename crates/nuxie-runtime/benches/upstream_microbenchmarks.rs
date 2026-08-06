//! Criterion mirrors of the pinned C++ geometry/path microbenchmarks.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use nuxie_render_api::{Mat2D as RenderMat2D, PathVerb, RawPath, Vec2D};
use nuxie_runtime::{
    Mat2D,
    upstream_microbenchmarks::{c_rand_max, measure_raw_path},
};

fn reset_cpp_rand() {
    // SAFETY: C's process-global PRNG accepts every unsigned seed. Benchmarks
    // run serially and mirror upstream's srand(0)/rand() construction.
    unsafe { libc::srand(0) };
}

fn cpp_rand() -> i32 {
    // SAFETY: `rand` has no preconditions. Benchmark execution is serial.
    unsafe { libc::rand() }
}

fn random_point(scale: f32) -> Vec2D {
    Vec2D::new(
        cpp_rand() as f32 * scale / c_rand_max(),
        cpp_rand() as f32 * scale / c_rand_max(),
    )
}

#[derive(Clone, Copy)]
enum BuildVerb {
    Move,
    Lines,
    Cubics,
    Close,
}

struct BuildRawPathWorkload {
    verbs: Vec<BuildVerb>,
    points: Vec<Vec2D>,
    path: RawPath,
}

impl BuildRawPathWorkload {
    fn new() -> Self {
        reset_cpp_rand();
        let mut verbs = Vec::with_capacity(100_000);
        let mut points = Vec::new();
        for _ in 0..100_000 {
            match cpp_rand() % 4 {
                0 => {
                    verbs.push(BuildVerb::Move);
                    points.push(random_point(100.0));
                }
                1 => {
                    verbs.push(BuildVerb::Lines);
                    points.extend((0..10).map(|_| random_point(100.0)));
                }
                2 => {
                    verbs.push(BuildVerb::Cubics);
                    points.extend((0..30).map(|_| random_point(100.0)));
                }
                _ => verbs.push(BuildVerb::Close),
            }
        }
        let mut workload = Self {
            verbs,
            points,
            path: RawPath::new(),
        };
        workload.run();
        workload
    }

    fn run(&mut self) -> usize {
        let mut point = 0;
        self.path.rewind();
        for verb in &self.verbs {
            match verb {
                BuildVerb::Move => {
                    let value = self.points[point];
                    point += 1;
                    self.path.move_to(value.x, value.y);
                }
                BuildVerb::Lines => {
                    for value in &self.points[point..point + 10] {
                        self.path.line_to(value.x, value.y);
                    }
                    point += 10;
                }
                BuildVerb::Cubics => {
                    for values in self.points[point..point + 30].chunks_exact(3) {
                        self.path.cubic_to(
                            values[0].x,
                            values[0].y,
                            values[1].x,
                            values[1].y,
                            values[2].x,
                            values[2].y,
                        );
                    }
                    point += 30;
                }
                BuildVerb::Close => self.path.close(),
            }
        }
        self.path.points().len()
    }
}

fn random_raw_path(scale: f32, iterations: usize, measure_shape: bool) -> RawPath {
    reset_cpp_rand();
    let mut path = RawPath::new();
    for _ in 0..iterations {
        let choice = cpp_rand() % 22;
        if measure_shape {
            if choice < 3 {
                let point = random_point(scale);
                path.line_to(point.x, point.y);
            } else {
                let p1 = random_point(scale);
                let p2 = random_point(scale);
                let p3 = random_point(scale);
                path.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
            }
        } else if choice == 0 {
            let point = random_point(scale);
            path.move_to(point.x, point.y);
        } else if choice == 1 {
            path.close();
        } else if choice < 12 {
            let point = random_point(scale);
            path.line_to(point.x, point.y);
        } else {
            let p1 = random_point(scale);
            let p2 = random_point(scale);
            let p3 = random_point(scale);
            path.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        }
    }
    path
}

fn iterate_raw_path(path: &RawPath) -> f32 {
    let mut point_index = 0;
    let mut current = Vec2D::new(0.0, 0.0);
    let mut start = current;
    let mut sum = [0.0f32; 4];
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                current = path.points()[point_index];
                point_index += 1;
                start = current;
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                point_index += 1;
                sum[0] += current.x * 0.5;
                sum[1] += current.y * 0.5;
                sum[2] += end.x * 0.5;
                sum[3] += end.y * 0.5;
                current = end;
            }
            PathVerb::Cubic => {
                let p1 = path.points()[point_index];
                let p2 = path.points()[point_index + 1];
                let p3 = path.points()[point_index + 2];
                point_index += 3;
                sum[0] += current.x * 0.125 + p2.x * 0.75;
                sum[1] += current.y * 0.125 + p2.y * 0.75;
                sum[2] += p1.x * 0.75 + p3.x * 0.125;
                sum[3] += p1.y * 0.75 + p3.y * 0.125;
                current = p3;
            }
            PathVerb::Close => {
                sum[0] += current.x * 0.5;
                sum[1] += current.y * 0.5;
                sum[2] += start.x * 0.5;
                sum[3] += start.y * 0.5;
                current = start;
            }
            PathVerb::Quad => unreachable!("upstream workload never emits quadratics"),
        }
    }
    sum.into_iter().sum()
}

struct MeasurePathWorkload {
    path: RawPath,
    matrix: RenderMat2D,
}

impl MeasurePathWorkload {
    fn new() -> Self {
        Self {
            path: random_raw_path(1000.0, 1_000_000, true),
            matrix: RenderMat2D([1.0, 0.0, 0.0, 1.0, -1.0, 1.0]),
        }
    }

    fn run(&mut self) -> f32 {
        self.matrix.0.swap(4, 5);
        let mut transformed = RawPath::new();
        transformed.add_path(&self.path, self.matrix);
        measure_raw_path(&transformed)
    }
}

struct MapPointsWorkload {
    matrix: Mat2D,
    points: Vec<(f32, f32)>,
    destination: Vec<(f32, f32)>,
}

impl MapPointsWorkload {
    fn new(matrix: Mat2D) -> Self {
        reset_cpp_rand();
        let points = (0..4096)
            .map(|_| {
                let point = random_point(100.0);
                (point.x, point.y)
            })
            .collect::<Vec<_>>();
        Self {
            destination: vec![(0.0, 0.0); points.len()],
            points,
            matrix,
        }
    }

    fn run(&mut self) -> f32 {
        for (destination, &(x, y)) in self.destination.iter_mut().zip(&self.points) {
            *destination = self.matrix.map_point(x, y);
        }
        for _ in 1..4096 {
            for index in 0..self.destination.len() {
                let (x, y) = self.destination[index];
                self.destination[index] = self.matrix.map_point(x, y);
            }
        }
        self.destination.last().map_or(0.0, |point| point.0)
    }
}

fn geometry_benches(criterion: &mut Criterion) {
    let mut build = BuildRawPathWorkload::new();
    criterion.bench_function("BuildRawPath", |bench| {
        bench.iter(|| black_box(build.run()));
    });

    let iterate = random_raw_path(100.0, 1_000_000, false);
    criterion.bench_function("IterateRawPath", |bench| {
        bench.iter(|| black_box(iterate_raw_path(black_box(&iterate))));
    });

    let mut measure = MeasurePathWorkload::new();
    criterion.bench_function("MeasurePath", |bench| {
        bench.iter(|| black_box(measure.run()));
    });

    let bounds = random_raw_path(100.0, 1_000_000, false);
    criterion.bench_function("RawPathBounds", |bench| {
        bench.iter(|| black_box(bounds.bounds()));
    });

    let mut scale_translate = MapPointsWorkload::new(Mat2D([-2.0, 0.0, 0.0, 3.0, -4.0, 5.0]));
    criterion.bench_function("MapPointsScaleTrans", |bench| {
        bench.iter(|| black_box(scale_translate.run()));
    });

    let mut affine = MapPointsWorkload::new(Mat2D([2.0, -3.0, -4.0, 5.0, 6.0, -7.0]));
    criterion.bench_function("MapPointsAffine", |bench| {
        bench.iter(|| black_box(affine.run()));
    });
}

criterion_group!(benches, geometry_benches);
criterion_main!(benches);
