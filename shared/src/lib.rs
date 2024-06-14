#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Time {
    pub elapsed: f32,
    pub frametime: f32,
}
