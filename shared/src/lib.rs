#![no_std]

#[repr(C)]
pub struct Time {
    pub elapsed: f32,
    pub frametime: f32,
}
