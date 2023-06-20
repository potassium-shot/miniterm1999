#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderParam {
    pub screen_size: [u32; 2],
    pub time: f32,
    _padding: u32,
}
