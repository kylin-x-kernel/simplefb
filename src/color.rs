/// Default foreground color (white)
pub const FG_COLOR: u32 = 0xFFFFFF;
/// Default background color (black)
pub const BG_COLOR: u32 = 0x000000;

/// ANSI color codes to RGB color mapping
const ANSI_COLORS: [u32; 16] = [
    0x000000, // 0: Black (30, 40)
    0xCC0000, // 1: Red (31, 41)
    0x00CC00, // 2: Green (32, 42)
    0xCCCC00, // 3: Yellow (33, 43)
    0x0000CC, // 4: Blue (34, 44)
    0xCC00CC, // 5: Magenta (35, 45)
    0x00CCCC, // 6: Cyan (36, 46)
    0xCCCCCC, // 7: White (37, 47)
    0x666666, // 8: Bright Black (90, 100)
    0xFF0000, // 9: Bright Red (91, 101)
    0x00FF00, // 10: Bright Green (92, 102)
    0xFFFF00, // 11: Bright Yellow (93, 103)
    0x0000FF, // 12: Bright Blue (94, 104)
    0xFF00FF, // 13: Bright Magenta (95, 105)
    0x00FFFF, // 14: Bright Cyan (96, 106)
    0xFFFFFF, // 15: Bright White (97, 107)
];

/// Convert ANSI color code to RGB color
pub fn ansi_to_rgb(code: u8) -> Option<u32> {
    match code {
        30..=37 => Some(ANSI_COLORS[(code - 30) as usize]),
        40..=47 => Some(ANSI_COLORS[(code - 40) as usize]),
        90..=97 => Some(ANSI_COLORS[(code - 90 + 8) as usize]),
        100..=107 => Some(ANSI_COLORS[(code - 100 + 8) as usize]),
        _ => None,
    }
}
