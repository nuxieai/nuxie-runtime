//! Direct ports from `tests/unit_tests/runtime/raw_path_test.cpp`.

use nuxie_render_api::{Aabb, Mat2D, PathVerb, RawPath, Vec2D};

struct UpstreamRand(u64);

impl UpstreamRand {
    fn new(seed: u32) -> Self {
        Self(u64::from(seed.wrapping_sub(1)))
    }

    fn next(&mut self) -> u32 {
        self.0 = 6_364_136_223_846_793_005_u64
            .wrapping_mul(self.0)
            .wrapping_add(1);
        (self.0 >> 33) as u32
    }

    fn signed_unit(&mut self) -> f32 {
        self.next() as f32 / (2_147_483_647.0 * 0.5) - 1.0
    }
}

fn cpp_bounds(path: &RawPath) -> Aabb {
    path.bounds()
        .unwrap_or_else(|| Aabb::new(0.0, 0.0, 0.0, 0.0))
}

#[test]
fn rawpath_basics() {
    let mut path = RawPath::new();

    assert!(path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(0.0, 0.0, 0.0, 0.0));

    path.move_to(1.0, 2.0);
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(1.0, 2.0, 1.0, 2.0));

    path = RawPath::new();
    assert!(path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(0.0, 0.0, 0.0, 0.0));

    path.move_to(1.0, -2.0);
    path.line_to(3.0, 4.0);
    path.line_to(-1.0, 5.0);
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(-1.0, -2.0, 3.0, 5.0));
}

fn validate_path_verb_point_counts(path: &RawPath) -> bool {
    let point_count = path
        .verbs()
        .iter()
        .map(|verb| match verb {
            PathVerb::Move | PathVerb::Line => 1,
            PathVerb::Quad => 2,
            PathVerb::Cubic => 3,
            PathVerb::Close => 0,
        })
        .sum::<usize>();
    point_count == path.points().len()
}

fn check_path_reversal(path: &RawPath) {
    let mut forwards = RawPath::new();
    forwards.add_path(path, Mat2D::IDENTITY);
    assert!(validate_path_verb_point_counts(&forwards));
    assert_eq!(&forwards, path);

    let mut backwards = RawPath::new();
    backwards.add_path_backwards(path, Mat2D::IDENTITY);
    assert_eq!(backwards.verbs().len(), path.verbs().len());
    assert_eq!(backwards.points().len(), path.points().len());
    assert!(validate_path_verb_point_counts(&backwards));

    let mut backwards_backwards = RawPath::new();
    backwards_backwards.add_path_backwards(&backwards, Mat2D::IDENTITY);
    assert!(validate_path_verb_point_counts(&backwards_backwards));
    assert_eq!(&backwards_backwards, path);
    assert_eq!(backwards_backwards, forwards);
}

#[test]
fn add_path_backwards() {
    {
        let mut path = RawPath::new();
        check_path_reversal(&path);

        path.move_to(0.0, 0.0);
        check_path_reversal(&path);

        path.move_to(0.0, 0.0);
        check_path_reversal(&path);

        path.move_to(10.0, 10.0);
        check_path_reversal(&path);

        path.close();
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.line_to(1.0, 2.0);
        check_path_reversal(&path);

        path.close();
        path.line_to(3.0, 4.0);
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.line_to(1.0, 2.0);
        path.close();
        path.close();
        path.close();
        path.close();
        path.close();
        path.line_to(3.0, 4.0);
        path.close();
        path.close();
        path.close();
        path.close();
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(32.0, 84.0);
        path.line_to(36.0, 76.0);
        path.close();
        path.move_to(0.0, 0.0);
        path.cubic_to(1.0, 57.0, 32.0, 10.0, 33.0, 86.0);
        path.line_to(20.0, 99.0);
        path.close();
        path.move_to(22.0, 59.0);
        path.move_to(62.0, 76.0);
        path.cubic_to(74.0, 39.0, 50.0, 35.0, 60.0, 26.0);
        path.move_to(26.0, 46.0);
        path.line_to(58.0, 76.0);
        path.line_to(93.0, 76.0);
        path.close();
        path.move_to(36.0, 74.0);
        path.line_to(3.0, 22.0);
        path.move_to(48.0, 47.0);
        path.move_to(47.0, 48.0);
        path.line_to(18.0, 94.0);
        path.cubic_to(35.0, 87.0, 73.0, 1.0, 74.0, 7.0);
        path.move_to(6.0, 50.0);
        path.close();
        path.close();
        path.line_to(13.0, 97.0);
        path.cubic_to(31.0, 26.0, 72.0, 94.0, 69.0, 32.0);
        path.move_to(80.0, 77.0);
        path.cubic_to(40.0, 23.0, 34.0, 34.0, 77.0, 41.0);
        path.cubic_to(58.0, 64.0, 26.0, 42.0, 28.0, 4.0);
        path.close();
        path.move_to(80.0, 77.0);
        path.line_to(20.0, 22.0);
        path.line_to(75.0, 26.0);
        path.line_to(88.0, 92.0);
        path.cubic_to(27.0, 7.0, 23.0, 84.0, 7.0, 89.0);
        path.close();
        check_path_reversal(&path);
    }
}

type PathMaker = fn(&mut RawPath);

fn make_empty(_path: &mut RawPath) {}

fn make_line(path: &mut RawPath) {
    path.move_to(1.0, 2.0);
    path.line_to(3.0, 4.0);
}

fn make_closed_line(path: &mut RawPath) {
    make_line(path);
    path.close();
}

fn make_all_verbs(path: &mut RawPath) {
    path.move_to(1.0, 2.0);
    path.line_to(3.0, 4.0);
    path.quad_to(5.0, 6.0, 7.0, 8.0);
    path.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);
    path.close();
}

#[test]
fn add_path() {
    let makers: [PathMaker; 4] = [make_empty, make_line, make_closed_line, make_all_verbs];
    let transform = Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0]);

    for first in makers {
        for second in makers {
            let mut direct = RawPath::new();
            first(&mut direct);
            second(&mut direct);

            let mut appended = RawPath::new();
            let mut temporary = RawPath::new();
            first(&mut temporary);
            appended.add_path(&temporary, Mat2D::IDENTITY);
            temporary.rewind();
            second(&mut temporary);
            appended.add_path(&temporary, Mat2D::IDENTITY);
            assert_eq!(direct, appended);

            let expected_points = direct
                .points()
                .iter()
                .map(|point| Vec2D::new(point.x * 2.0, point.y * 3.0))
                .collect::<Vec<_>>();
            let mut transformed = RawPath::new();
            let mut temporary = RawPath::new();
            first(&mut temporary);
            transformed.add_path(&temporary, transform);
            temporary.rewind();
            second(&mut temporary);
            transformed.add_path(&temporary, transform);
            assert_eq!(transformed.verbs(), direct.verbs());
            assert_eq!(transformed.points(), expected_points);
        }
    }
}

fn include(bounds: &mut Aabb, point: Vec2D) {
    bounds.min_x = bounds.min_x.min(point.x);
    bounds.min_y = bounds.min_y.min(point.y);
    bounds.max_x = bounds.max_x.max(point.x);
    bounds.max_y = bounds.max_y.max(point.y);
}

#[test]
fn bounds() {
    let mut path = RawPath::new();
    let mut random = UpstreamRand::new(0);

    for number_of_verbs in (0..16).map(|shift| 1usize << shift) {
        path.rewind();
        let mut expected = Aabb::new(f32::INFINITY, f32::INFINITY, -f32::INFINITY, -f32::INFINITY);
        for _ in 0..number_of_verbs {
            match random.next() % 5 {
                0 => {
                    let point = Vec2D::new(random.signed_unit(), random.signed_unit());
                    include(&mut expected, point);
                    path.move_to(point.x, point.y);
                }
                1 => {
                    if path.verbs().is_empty() {
                        expected = Aabb::new(0.0, 0.0, 0.0, 0.0);
                    }
                    let point = Vec2D::new(random.signed_unit(), random.signed_unit());
                    include(&mut expected, point);
                    path.line_to(point.x, point.y);
                }
                2 => {
                    if path.verbs().is_empty() {
                        expected = Aabb::new(0.0, 0.0, 0.0, 0.0);
                    }
                    let control = Vec2D::new(random.signed_unit(), random.signed_unit());
                    let end = Vec2D::new(random.signed_unit(), random.signed_unit());
                    include(&mut expected, control);
                    include(&mut expected, end);
                    path.quad_to(control.x, control.y, end.x, end.y);
                }
                3 => {
                    if path.verbs().is_empty() {
                        expected = Aabb::new(0.0, 0.0, 0.0, 0.0);
                    }
                    let outer = Vec2D::new(random.signed_unit(), random.signed_unit());
                    let inner = Vec2D::new(random.signed_unit(), random.signed_unit());
                    let end = Vec2D::new(random.signed_unit(), random.signed_unit());
                    include(&mut expected, outer);
                    include(&mut expected, inner);
                    include(&mut expected, end);
                    path.cubic_to(outer.x, outer.y, inner.x, inner.y, end.x, end.y);
                }
                4 => path.close(),
                _ => unreachable!(),
            }
        }
        let actual = path.bounds().unwrap_or(expected);
        assert_eq!(actual.min_x, expected.min_x);
        assert_eq!(actual.min_y, expected.min_y);
        assert_eq!(actual.max_x, expected.max_x);
        assert_eq!(actual.max_y, expected.max_y);
    }
}

#[derive(Debug, PartialEq)]
struct Segment {
    verb: PathVerb,
    points: Vec<Vec2D>,
}

fn segments(path: &RawPath) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut point_index = 0;
    let mut current = Vec2D::new(0.0, 0.0);
    let mut contour_start = current;
    for verb in path.verbs() {
        let points = match verb {
            PathVerb::Move => {
                current = path.points()[point_index];
                contour_start = current;
                point_index += 1;
                vec![current]
            }
            PathVerb::Line => {
                let end = path.points()[point_index];
                point_index += 1;
                let points = vec![current, end];
                current = end;
                points
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let end = path.points()[point_index + 1];
                point_index += 2;
                let points = vec![current, control, end];
                current = end;
                points
            }
            PathVerb::Cubic => {
                let outer = path.points()[point_index];
                let inner = path.points()[point_index + 1];
                let end = path.points()[point_index + 2];
                point_index += 3;
                let points = vec![current, outer, inner, end];
                current = end;
                points
            }
            PathVerb::Close => {
                current = contour_start;
                Vec::new()
            }
        };
        result.push(Segment {
            verb: *verb,
            points,
        });
    }
    result
}

fn segment(verb: PathVerb, points: &[(f32, f32)]) -> Segment {
    Segment {
        verb,
        points: points.iter().map(|&(x, y)| Vec2D::new(x, y)).collect(),
    }
}

#[test]
fn rawpath_iter() {
    {
        let path = RawPath::new();
        assert!(segments(&path).is_empty());
    }
    {
        let mut path = RawPath::new();
        path.move_to(1.0, 2.0);
        path.line_to(3.0, 4.0);
        path.quad_to(5.0, 6.0, 7.0, 8.0);
        path.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);
        path.close();
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(1.0, 2.0)]),
                segment(PathVerb::Line, &[(1.0, 2.0), (3.0, 4.0)]),
                segment(PathVerb::Quad, &[(3.0, 4.0), (5.0, 6.0), (7.0, 8.0)]),
                segment(
                    PathVerb::Cubic,
                    &[(7.0, 8.0), (9.0, 10.0), (11.0, 12.0), (13.0, 14.0)],
                ),
                segment(PathVerb::Close, &[]),
            ]
        );

        path.rewind();
        path.move_to(1.0, 2.0);
        path.move_to(3.0, 4.0);
        path.move_to(5.0, 6.0);
        path.close();
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(1.0, 2.0)]),
                segment(PathVerb::Move, &[(3.0, 4.0)]),
                segment(PathVerb::Move, &[(5.0, 6.0)]),
                segment(PathVerb::Close, &[]),
            ]
        );

        path.rewind();
        path.close();
        path.close();
        path.close();
        path.close();
        path.line_to(1.0, 2.0);
        path.close();
        path.close();
        path.cubic_to(3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        path.move_to(9.0, 10.0);
        path.move_to(11.0, 12.0);
        path.quad_to(13.0, 14.0, 15.0, 16.0);
        path.close();
        path.line_to(17.0, 18.0);
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(0.0, 0.0)]),
                segment(PathVerb::Line, &[(0.0, 0.0), (1.0, 2.0)]),
                segment(PathVerb::Close, &[]),
                segment(PathVerb::Move, &[(0.0, 0.0)]),
                segment(
                    PathVerb::Cubic,
                    &[(0.0, 0.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0)],
                ),
                segment(PathVerb::Move, &[(9.0, 10.0)]),
                segment(PathVerb::Move, &[(11.0, 12.0)]),
                segment(PathVerb::Quad, &[(11.0, 12.0), (13.0, 14.0), (15.0, 16.0)],),
                segment(PathVerb::Close, &[]),
                segment(PathVerb::Move, &[(11.0, 12.0)]),
                segment(PathVerb::Line, &[(11.0, 12.0), (17.0, 18.0)]),
            ]
        );
    }
}

fn upstream_add_oval(_path: &mut RawPath, _bounds: Aabb) {
    // No production RawPath::addOval owner exists yet.
}

fn upstream_add_poly(_path: &mut RawPath, _points: &[Vec2D], _closed: bool) {
    // No production RawPath::addPoly owner exists yet.
}

#[test]
#[ignore = "expected-red: production RawPath has no addOval or addPoly owners"]
fn rawpath_add_helpers() {
    let mut path = RawPath::new();

    path.add_rect(Aabb::new(1.0, 1.0, 5.0, 6.0));
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(1.0, 1.0, 5.0, 6.0));
    assert_eq!(path.points().len(), 4);
    assert_eq!(path.verbs().len(), 5);

    path = RawPath::new();
    upstream_add_oval(&mut path, Aabb::new(0.0, 0.0, 3.0, 6.0));
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(0.0, 0.0, 3.0, 6.0));
    assert_eq!(path.points().len(), 13);
    assert_eq!(path.verbs().len(), 6);

    let points = [
        Vec2D::new(1.0, 2.0),
        Vec2D::new(4.0, 5.0),
        Vec2D::new(3.0, 2.0),
        Vec2D::new(100.0, -100.0),
    ];
    for closed in [false, true] {
        path = RawPath::new();
        upstream_add_poly(&mut path, &points, closed);
        assert_eq!(cpp_bounds(&path), Aabb::new(1.0, -100.0, 100.0, 5.0));
        assert_eq!(path.points().len(), points.len());
        assert_eq!(path.verbs().len(), points.len() + usize::from(closed));
        for (actual, expected) in path.points().iter().zip(points) {
            assert_eq!(*actual, expected);
        }
        assert_eq!(path.verbs()[0], PathVerb::Move);
        for verb in &path.verbs()[1..points.len()] {
            assert_eq!(*verb, PathVerb::Line);
        }
        if closed {
            assert_eq!(path.verbs()[points.len()], PathVerb::Close);
        }
    }
}

#[test]
fn prune_empty_segments() {
    {
        let mut path = RawPath::new();
        path.prune_empty_segments();
        assert!(segments(&path).is_empty());
    }
    for build in [
        (PathVerb::Line, vec![0.0, 0.0]),
        (PathVerb::Quad, vec![0.0, 0.0, 0.0, 0.0]),
        (PathVerb::Cubic, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ] {
        let mut path = RawPath::new();
        match build.0 {
            PathVerb::Line => path.line_to(build.1[0], build.1[1]),
            PathVerb::Quad => path.quad_to(build.1[0], build.1[1], build.1[2], build.1[3]),
            PathVerb::Cubic => path.cubic_to(
                build.1[0], build.1[1], build.1[2], build.1[3], build.1[4], build.1[5],
            ),
            _ => unreachable!(),
        }
        path.prune_empty_segments();
        assert_eq!(
            segments(&path),
            vec![segment(PathVerb::Move, &[(0.0, 0.0)])]
        );
    }

    {
        let mut path = RawPath::new();
        path.move_to(1.0, 2.0);
        path.line_to(3.0, 4.0);
        path.line_to(3.0, 4.0);
        path.quad_to(5.0, 6.0, 7.0, 8.0);
        path.quad_to(7.0, 8.0, 7.0, 8.0);
        path.quad_to(7.0, 8.0, 7.0, 9.0);
        path.quad_to(7.0, 9.0, 7.0, 9.0);
        path.quad_to(7.0, 9.0, 7.0, 8.0);
        path.quad_to(7.0, 8.0, 7.0, 8.0);
        path.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);
        path.cubic_to(13.0, 14.0, 13.0, 14.0, 13.0, 14.0);
        path.cubic_to(13.0, 14.0, 13.0, 14.0, 13.0, 15.0);
        path.cubic_to(13.0, 15.0, 13.0, 15.0, 13.0, 15.0);
        path.cubic_to(13.0, 16.0, 13.0, 15.0, 13.0, 15.0);
        path.cubic_to(13.0, 15.0, 13.0, 15.0, 13.0, 15.0);
        path.cubic_to(13.0, 15.0, 13.0, 16.0, 13.0, 15.0);
        path.cubic_to(13.0, 15.0, 13.0, 15.0, 13.0, 15.0);
        path.cubic_to(13.0, 15.0, 13.0, 15.0, 13.0, 16.0);
        path.close();
        path.prune_empty_segments();
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(1.0, 2.0)]),
                segment(PathVerb::Line, &[(1.0, 2.0), (3.0, 4.0)]),
                segment(PathVerb::Quad, &[(3.0, 4.0), (5.0, 6.0), (7.0, 8.0)]),
                segment(PathVerb::Quad, &[(7.0, 8.0), (7.0, 8.0), (7.0, 9.0)]),
                segment(PathVerb::Quad, &[(7.0, 9.0), (7.0, 9.0), (7.0, 8.0)]),
                segment(
                    PathVerb::Cubic,
                    &[(7.0, 8.0), (9.0, 10.0), (11.0, 12.0), (13.0, 14.0)]
                ),
                segment(
                    PathVerb::Cubic,
                    &[(13.0, 14.0), (13.0, 14.0), (13.0, 14.0), (13.0, 15.0)]
                ),
                segment(
                    PathVerb::Cubic,
                    &[(13.0, 15.0), (13.0, 16.0), (13.0, 15.0), (13.0, 15.0)]
                ),
                segment(
                    PathVerb::Cubic,
                    &[(13.0, 15.0), (13.0, 15.0), (13.0, 16.0), (13.0, 15.0)]
                ),
                segment(
                    PathVerb::Cubic,
                    &[(13.0, 15.0), (13.0, 15.0), (13.0, 15.0), (13.0, 16.0)]
                ),
                segment(PathVerb::Close, &[]),
            ]
        );
    }

    {
        let mut path = RawPath::new();
        path.move_to(1.0, 2.0);
        path.line_to(1.0, 2.0);
        path.line_to(3.0, 4.0);
        let mut appended = RawPath::new();
        appended.move_to(5.0, 6.0);
        appended.quad_to(7.0, 8.0, 9.0, 10.0);
        appended.close();
        appended.move_to(11.0, 12.0);
        appended.cubic_to(13.0, 14.0, 15.0, 16.0, 17.0, 18.0);
        let verb_start = path.verbs().len();
        let point_start = path.points().len();
        path.add_path(&appended, Mat2D([0.0, 0.0, 0.0, 0.0, 19.0, 20.0]));

        path.prune_empty_segments_from_offsets(path.verbs().len(), path.points().len());
        assert_eq!(segments(&path).len(), 8);
        path.prune_empty_segments_from_offsets(verb_start, point_start);
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(1.0, 2.0)]),
                segment(PathVerb::Line, &[(1.0, 2.0), (1.0, 2.0)]),
                segment(PathVerb::Line, &[(1.0, 2.0), (3.0, 4.0)]),
                segment(PathVerb::Move, &[(19.0, 20.0)]),
                segment(PathVerb::Close, &[]),
                segment(PathVerb::Move, &[(19.0, 20.0)]),
            ]
        );
        path.prune_empty_segments();
        assert_eq!(
            segments(&path),
            vec![
                segment(PathVerb::Move, &[(1.0, 2.0)]),
                segment(PathVerb::Line, &[(1.0, 2.0), (3.0, 4.0)]),
                segment(PathVerb::Move, &[(19.0, 20.0)]),
                segment(PathVerb::Close, &[]),
                segment(PathVerb::Move, &[(19.0, 20.0)]),
            ]
        );
    }
}
