#![no_std]


use spirv_std::glam::{Vec4,Vec2,Vec4Swizzles, Mat4};
use spirv_std::spirv;
use shared::Time;

const POINT_SIZE: f32 = 2.0;

const A: f32 = 1.89;



#[spirv(vertex)]
pub fn main_vs(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera_view_proj: &Mat4,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] time: &Time,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] points: &mut [Vec4],
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    out_d: &mut Vec4,
) {
    let vi = vertex_index as usize;

    let mut d = (-A * points[vi].xyz() - 4.0 * points[vi].yzx() - 4.0 * points[vi].zxy() - (points[vi].yzx() * points[vi].yzx())).extend(0.0);
    d *= time.frametime;

    let dpos = points[vi] + d;
    points[vi] = dpos;

    *out_pos = *camera_view_proj * dpos;
    *out_point_size = POINT_SIZE;
    d.w = 0.0;
    *out_d = d;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(point_coord)] _pos: Vec2,
    d: Vec4,
    output: &mut Vec4,
) {
    *output = 1.0 / 1.0 - d;
}
