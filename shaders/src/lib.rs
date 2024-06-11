#![no_std]

use spirv_std::glam::{Vec4, Vec3, Mat4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn main_vs(
    pos: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera_view_proj: &Mat4,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] time: &f32,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
) {
    let y_rot = Mat4::from_rotation_y((core::f32::consts::PI / 45.0) * time * 2.0);

    *out_pos = *camera_view_proj * y_rot * pos;
    *out_point_size = 5.0;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(point_coord)] pos: Vec4,
    output: &mut Vec4,
) {
    *output = Vec4::new(1.0, 0.0, 1.0, 1.0);
}

