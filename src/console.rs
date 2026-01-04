extern crate alloc;

use font8x8::{UnicodeFonts, BASIC_FONTS};
use crate::buffer::LogBuffer;
use crate::config::FramebufferConfig;
use crate::color::{FG_COLOR, BG_COLOR, ansi_to_rgb};

/// Base font dimensions (8x8 font)
const BASE_FONT_WIDTH: usize = 8;
const BASE_FONT_HEIGHT: usize = 8;

/// ANSI escape sequence parser state
#[derive(Clone, Copy, PartialEq)]
enum AnsiState {
    Normal,
    Escape,      // After ESC (\x1B)
    Csi,         // After ESC [
}

/// SimpleFb Console structure with configurable font size and log buffer
pub struct SimpleFbConsole {
    config: FramebufferConfig,
    cursor_x: usize,
    cursor_y: usize,
    max_cols: usize,
    max_rows: usize,
    fg_color: u32,
    bg_color: u32,
    default_fg_color: u32,
    default_bg_color: u32,
    log_buffer: LogBuffer,
    // ANSI escape sequence parser state
    ansi_state: AnsiState,
    ansi_param: u8,
}

impl SimpleFbConsole {
    /// Creates a new SimpleFb console with specified configuration and log buffer
    pub fn new(config: FramebufferConfig, log_buffer: LogBuffer) -> Self {
        let font_height = if config.font_height == 0 { BASE_FONT_HEIGHT } else { config.font_height };
        // Assume square pixels for now, or maintain aspect ratio if needed.
        // For 8x8 font, width = height usually.
        let font_width = font_height; 
        
        // Update config with validated height
        let mut config = config;
        config.font_height = font_height;

        Self {
            config,
            cursor_x: 0,
            cursor_y: 0,
            max_cols: config.width / font_width,
            max_rows: config.height / font_height,
            fg_color: FG_COLOR,
            bg_color: BG_COLOR,
            default_fg_color: FG_COLOR,
            default_bg_color: BG_COLOR,
            log_buffer,
            ansi_state: AnsiState::Normal,
            ansi_param: 0,
        }
    }

    /// Returns the current font width in pixels
    fn font_width(&self) -> usize {
        self.config.font_height // Assuming square font
    }

    /// Returns the current font height in pixels
    fn font_height(&self) -> usize {
        self.config.font_height
    }

    /// Draws a single pixel at (x, y) with the specified color
    fn draw_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= self.config.width || y >= self.config.height {
            return;
        }
        unsafe {
            let offset = y * self.config.width + x;
            core::ptr::write_volatile((self.config.base_addr as *mut u32).add(offset), color);
        }
    }

    /// Draws a character at (x, y) with foreground and background colors
    /// Uses Nearest Neighbor Interpolation for arbitrary scaling
    fn draw_char(&mut self, ch: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        let glyph = BASIC_FONTS.get(ch).unwrap_or(BASIC_FONTS.get('?').unwrap());
        
        let target_size = self.font_height();
        
        for dy in 0..target_size {
            for dx in 0..target_size {
                // Nearest neighbor interpolation
                // Map target pixel (dx, dy) to source pixel (src_x, src_y) in 8x8 grid
                let src_x = dx * BASE_FONT_WIDTH / target_size;
                let src_y = dy * BASE_FONT_HEIGHT / target_size;
                
                let byte = glyph[src_y];
                let is_set = (byte & (1 << src_x)) != 0;
                
                let color = if is_set { fg_color } else { bg_color };
                self.draw_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Scrolls the screen up by one line
    fn scroll_up(&mut self) {
        let font_height = self.font_height();
        
        unsafe {
            let ptr = self.config.base_addr as *mut u32;
            let row_pixels = font_height * self.config.width;
            let total_pixels = self.config.height * self.config.width;
            let move_pixels = total_pixels - row_pixels;
            
            // Move all content up by one line
            core::ptr::copy(ptr.add(row_pixels), ptr, move_pixels);
            
            // Clear the bottom line
            let bottom_ptr = ptr.add(move_pixels);
            for i in 0..row_pixels {
                core::ptr::write_volatile(bottom_ptr.add(i), self.bg_color);
            }
        }
    }

    /// Clears the entire screen
    pub fn clear(&mut self) {
        unsafe {
            let ptr = self.config.base_addr as *mut u32;
            let total_pixels = self.config.width * self.config.height;
            for i in 0..total_pixels {
                core::ptr::write_volatile(ptr.add(i), self.bg_color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Process ANSI SGR (Select Graphic Rendition) parameter
    fn process_ansi_sgr(&mut self, param: u8) {
        match param {
            0 => {
                // Reset to default
                self.fg_color = self.default_fg_color;
                self.bg_color = self.default_bg_color;
            }
            1 => {
                // Bold - we can make the color brighter
                // For simplicity, just keep current color
            }
            30..=37 | 90..=97 => {
                // Foreground color
                if let Some(color) = ansi_to_rgb(param) {
                    self.fg_color = color;
                }
            }
            40..=47 | 100..=107 => {
                // Background color
                if let Some(color) = ansi_to_rgb(param) {
                    self.bg_color = color;
                }
            }
            39 => {
                // Default foreground color
                self.fg_color = self.default_fg_color;
            }
            49 => {
                // Default background color
                self.bg_color = self.default_bg_color;
            }
            _ => {
                // Unsupported SGR parameter, ignore
            }
        }
    }

    /// Writes a single byte to the console with ANSI escape sequence support
    pub fn write_byte(&mut self, byte: u8) {
        // Cache to log buffer
        self.log_buffer.push(byte);
        
        match self.ansi_state {
            AnsiState::Normal => {
                match byte {
                    0x1B => {
                        // ESC character - start escape sequence
                        self.ansi_state = AnsiState::Escape;
                    }
                    b'\n' => {
                        self.new_line();
                    }
                    b'\r' => {
                        self.cursor_x = 0;
                    }
                    b'\t' => {
                        // Handle tab as 4 spaces
                        let spaces = 4 - (self.cursor_x % 4);
                        for _ in 0..spaces {
                            self.write_visible_char(b' ');
                        }
                    }
                    _ => {
                        self.write_visible_char(byte);
                    }
                }
            }
            AnsiState::Escape => {
                match byte {
                    b'[' => {
                        // CSI (Control Sequence Introducer)
                        self.ansi_state = AnsiState::Csi;
                        self.ansi_param = 0;
                    }
                    _ => {
                        // Unknown escape sequence, return to normal
                        self.ansi_state = AnsiState::Normal;
                    }
                }
            }
            AnsiState::Csi => {
                match byte {
                    b'0'..=b'9' => {
                        // Accumulate numeric parameter
                        self.ansi_param = self.ansi_param.saturating_mul(10).saturating_add(byte - b'0');
                    }
                    b';' => {
                        // Parameter separator - process current parameter and continue
                        self.process_ansi_sgr(self.ansi_param);
                        self.ansi_param = 0;
                    }
                    b'm' => {
                        // SGR (Select Graphic Rendition) - end of sequence
                        self.process_ansi_sgr(self.ansi_param);
                        self.ansi_state = AnsiState::Normal;
                        self.ansi_param = 0;
                    }
                    _ => {
                        // Unknown CSI sequence, return to normal
                        self.ansi_state = AnsiState::Normal;
                        self.ansi_param = 0;
                    }
                }
            }
        }
    }

    /// Writes a visible character to the screen
    fn write_visible_char(&mut self, byte: u8) {
        if self.cursor_x >= self.max_cols {
            self.new_line();
        }
        
        let ch = byte as char;
        let x = self.cursor_x * self.font_width();
        let y = self.cursor_y * self.font_height();
        self.draw_char(ch, x, y, self.fg_color, self.bg_color);
        self.cursor_x += 1;
    }

    /// Moves to a new line, scrolling if necessary
    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.max_rows {
            self.scroll_up();
            self.cursor_y = self.max_rows - 1;
        }
    }

    /// Writes a slice of bytes to the console
    pub fn write_bytes(&mut self, s: &[u8]) {
        for &b in s {
            self.write_byte(b);
        }
    }

    /// Sets the font height in pixels
    pub fn set_font_height(&mut self, height: usize) {
        let height = if height == 0 { BASE_FONT_HEIGHT } else { height };
        self.config.font_height = height;
        self.max_cols = self.config.width / self.font_width();
        self.max_rows = self.config.height / self.font_height();
        
        // Adjust cursor if it's now out of bounds
        if self.cursor_x >= self.max_cols {
            self.cursor_x = self.max_cols - 1;
        }
        if self.cursor_y >= self.max_rows {
            self.cursor_y = self.max_rows - 1;
        }
    }

    /// Sets the foreground color
    pub fn set_fg_color(&mut self, color: u32) {
        self.fg_color = color;
    }

    /// Sets the background color
    pub fn set_bg_color(&mut self, color: u32) {
        self.bg_color = color;
    }

    /// Returns the number of cached log bytes
    pub fn log_buffer_len(&self) -> usize {
        self.log_buffer.len()
    }

    /// Redraws the screen from the log buffer (useful after font size change)
    /// This properly handles ANSI escape sequences
    pub fn redraw_from_log(&mut self) {
        // Reset screen and colors
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.fg_color = self.default_fg_color;
        self.bg_color = self.default_bg_color;
        self.ansi_state = AnsiState::Normal;
        self.ansi_param = 0;
        
        // Clear screen
        unsafe {
            let ptr = self.config.base_addr as *mut u32;
            let total_pixels = self.config.width * self.config.height;
            for i in 0..total_pixels {
                core::ptr::write_volatile(ptr.add(i), self.default_bg_color);
            }
        }
        
        // Collect log buffer content
        let bytes: alloc::vec::Vec<u8> = self.log_buffer.iter().collect();
        
        // Replay all bytes with ANSI support but without re-buffering
        for &b in &bytes {
            match self.ansi_state {
                AnsiState::Normal => {
                    match b {
                        0x1B => {
                            self.ansi_state = AnsiState::Escape;
                        }
                        b'\n' => {
                            self.new_line();
                        }
                        b'\r' => {
                            self.cursor_x = 0;
                        }
                        b'\t' => {
                            let spaces = 4 - (self.cursor_x % 4);
                            for _ in 0..spaces {
                                self.write_visible_char(b' ');
                            }
                        }
                        _ => {
                            self.write_visible_char(b);
                        }
                    }
                }
                AnsiState::Escape => {
                    match b {
                        b'[' => {
                            self.ansi_state = AnsiState::Csi;
                            self.ansi_param = 0;
                        }
                        _ => {
                            self.ansi_state = AnsiState::Normal;
                        }
                    }
                }
                AnsiState::Csi => {
                    match b {
                        b'0'..=b'9' => {
                            self.ansi_param = self.ansi_param.saturating_mul(10).saturating_add(b - b'0');
                        }
                        b';' => {
                            self.process_ansi_sgr(self.ansi_param);
                            self.ansi_param = 0;
                        }
                        b'm' => {
                            self.process_ansi_sgr(self.ansi_param);
                            self.ansi_state = AnsiState::Normal;
                            self.ansi_param = 0;
                        }
                        _ => {
                            self.ansi_state = AnsiState::Normal;
                            self.ansi_param = 0;
                        }
                    }
                }
            }
        }
    }
}
