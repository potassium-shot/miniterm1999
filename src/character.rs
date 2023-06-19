#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Character {
    pub bg: [f32; 3],
    _padding: u32,
    pub fg: [f32; 3],
    pub code: u32,
}

impl Character {
    pub fn new(bg: [f32; 3], fg: [f32; 3], code: u32) -> Self {
        Self {
            bg,
            _padding: 0,
            fg,
            code,
        }
    }
}
