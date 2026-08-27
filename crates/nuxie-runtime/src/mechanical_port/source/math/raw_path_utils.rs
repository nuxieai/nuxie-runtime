use super::vec2d::Vec2D;

fn two(value: Vec2D) -> Vec2D {
    value + value
}

#[derive(Clone, Copy, Debug)]
pub struct EvalQuad {
    pub a: Vec2D,
    pub b: Vec2D,
    pub c: Vec2D,
}
impl EvalQuad {
    pub fn new(points: &[Vec2D; 3]) -> Self {
        Self {
            a: points[0] - two(points[1]) + points[2],
            b: two(points[1] - points[0]),
            c: points[0],
        }
    }
    pub fn at(self, t: f32) -> Vec2D {
        let value = Vec2D::scale_and_add(self.b, self.a, t);
        Vec2D::scale_and_add(self.c, value, t)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EvalCubic {
    pub a: Vec2D,
    pub b: Vec2D,
    pub c: Vec2D,
    pub d: Vec2D,
}
impl EvalCubic {
    pub fn new(points: &[Vec2D; 4]) -> Self {
        Self {
            a: points[3] + 3.0 * (points[1] - points[2]) - points[0],
            b: 3.0 * (points[2] - two(points[1]) + points[0]),
            c: 3.0 * (points[1] - points[0]),
            d: points[0],
        }
    }
    pub fn at(self, t: f32) -> Vec2D {
        let value = Vec2D::scale_and_add(self.b, self.a, t);
        let value = Vec2D::scale_and_add(self.c, value, t);
        Vec2D::scale_and_add(self.d, value, t)
    }
}

fn lerp(a: Vec2D, b: Vec2D, t: f32) -> Vec2D {
    Vec2D::new(a.x.mul_add(1.0 - t, b.x * t), a.y.mul_add(1.0 - t, b.y * t))
}

pub fn quad_subdivide(src: &[Vec2D; 3], t: f32, dst: &mut [Vec2D; 5]) {
    assert!((0.0..=1.0).contains(&t));
    let ab = lerp(src[0], src[1], t);
    let bc = lerp(src[1], src[2], t);
    dst[0] = src[0];
    dst[1] = ab;
    dst[2] = lerp(ab, bc, t);
    dst[3] = bc;
    dst[4] = src[2];
}
pub fn cubic_subdivide(src: &[Vec2D; 4], t: f32, dst: &mut [Vec2D; 7]) {
    assert!((0.0..=1.0).contains(&t));
    let ab = lerp(src[0], src[1], t);
    let bc = lerp(src[1], src[2], t);
    let cd = lerp(src[2], src[3], t);
    let abc = lerp(ab, bc, t);
    let bcd = lerp(bc, cd, t);
    dst[0] = src[0];
    dst[1] = ab;
    dst[2] = abc;
    dst[3] = lerp(abc, bcd, t);
    dst[4] = bcd;
    dst[5] = cd;
    dst[6] = src[3];
}
pub fn line_extract(src: &[Vec2D; 2], start_t: f32, end_t: f32, dst: &mut [Vec2D; 2]) {
    assert!(start_t <= end_t && start_t >= 0.0 && end_t <= 1.0);
    dst[0] = lerp(src[0], src[1], start_t);
    dst[1] = lerp(src[0], src[1], end_t);
}
pub fn quad_extract(src: &[Vec2D; 3], start_t: f32, end_t: f32, dst: &mut [Vec2D; 3]) {
    assert!(start_t <= end_t && start_t >= 0.0 && end_t <= 1.0);
    let mut tmp = [Vec2D::default(); 5];
    if start_t == 0.0 && end_t == 1.0 {
        dst.copy_from_slice(src);
    } else if start_t == 0.0 {
        quad_subdivide(src, end_t, &mut tmp);
        dst.copy_from_slice(&tmp[..3]);
    } else if end_t == 1.0 {
        quad_subdivide(src, start_t, &mut tmp);
        dst.copy_from_slice(&tmp[2..5]);
    } else {
        assert!(end_t > 0.0);
        quad_subdivide(src, end_t, &mut tmp);
        let mut tmp2 = [Vec2D::default(); 5];
        quad_subdivide((&tmp[..3]).try_into().unwrap(), start_t / end_t, &mut tmp2);
        dst.copy_from_slice(&tmp2[2..5]);
    }
}
pub fn cubic_extract(src: &[Vec2D; 4], start_t: f32, end_t: f32, dst: &mut [Vec2D; 4]) {
    assert!(start_t <= end_t && start_t >= 0.0 && end_t <= 1.0);
    let mut tmp = [Vec2D::default(); 7];
    if start_t == 0.0 && end_t == 1.0 {
        dst.copy_from_slice(src);
    } else if start_t == 0.0 {
        cubic_subdivide(src, end_t, &mut tmp);
        dst.copy_from_slice(&tmp[..4]);
    } else if end_t == 1.0 {
        cubic_subdivide(src, start_t, &mut tmp);
        dst.copy_from_slice(&tmp[3..7]);
    } else {
        assert!(end_t > 0.0);
        cubic_subdivide(src, end_t, &mut tmp);
        let mut tmp2 = [Vec2D::default(); 7];
        cubic_subdivide((&tmp[..4]).try_into().unwrap(), start_t / end_t, &mut tmp2);
        dst.copy_from_slice(&tmp2[3..7]);
    }
}
