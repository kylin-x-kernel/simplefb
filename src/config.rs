#[derive(Debug, Clone, Copy)]
pub struct FramebufferConfig {
    pub base_addr: usize,
    pub width: usize,
    pub height: usize,
    /// Target font height in pixels (e.g., 16 for 16x16 characters)
    pub font_height: usize,
}
