#![no_std]

use spirv_std::glam::{Vec4,Vec2, Mat4};
use spirv_std::spirv;

const A: f32 = 1.89;
const DT: f32 = 0.01;
const POINT_SIZE: f32 = 2.0;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera_view_proj: &Mat4,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] _time: &f32,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] points: &mut [Vec4],
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    out_dx: &mut f32,
    out_dy: &mut f32,
    out_dz: &mut f32,
) {
    let vi = vertex_index as usize;
    let mut dx = -A * points[vi].x - 4.0 * points[vi].y - 4.0 * points[vi].z - (points[vi].y * points[vi].y);
    dx *=DT;
    let mut dy = -A * points[vi].y - 4.0 * points[vi].z - 4.0 * points[vi].x - (points[vi].z * points[vi].z);
    dy *=DT;
    let mut dz = -A * points[vi].z - 4.0 * points[vi].x - 4.0 * points[vi].y - (points[vi].x * points[vi].x);
    dz *=DT;
    let dpos = Vec4::new(points[vi].x + dx, points[vi].y + dy, points[vi].z + dz, 1.0);
    points[vi] = dpos;

    *out_pos = *camera_view_proj * dpos;
    *out_point_size = POINT_SIZE;
    *out_dx = dx;
    *out_dy = dy;
    *out_dz = dz;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(point_coord)] _pos: Vec2,
    dx: f32,
    dy: f32,
    dz: f32,
    output: &mut Vec4,
) {
    *output = Vec4::new(
        1.0 / (1.0 - dx),
        1.0 / (1.0 - dy),
        1.0 / (1.0 - dz),
        1.0,
    );
}


