extern crate alloc;

use crate::config::FramebufferConfig;

/// Draws a picture at the specified coordinates
/// 
/// `data` should be a slice of u32 colors (0x00RRGGBB).
pub fn draw_picture(config: &FramebufferConfig, x: usize, y: usize, width: usize, height: usize, data: &[u32]) {
    if data.len() < width * height {
        return;
    }

    unsafe {
        let ptr = config.base_addr as *mut u32;
        
        for row in 0..height {
            for col in 0..width {
                let screen_x = x + col;
                let screen_y = y + row;
                
                if screen_x < config.width && screen_y < config.height {
                    let color = data[row * width + col];
                    let offset = screen_y * config.width + screen_x;
                    core::ptr::write_volatile(ptr.add(offset), color);
                }
            }
        }
    }
}
