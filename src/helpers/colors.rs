/// Color palette based on Catppuccin theme (matching your Lua config)
pub struct Colors;

impl Colors {
	pub const BASE: u32 = 0xff1e1e2e;
	// Accent colors
	pub const BLUE: u32 = 0xff89b4fa;
	pub const CRUST: u32 = 0xff11111b;
	pub const FLAMINGO: u32 = 0xfff2cdcd;
	pub const GREEN: u32 = 0xffa6e3a1;
	pub const LAVENDER: u32 = 0xffb4befe;
	pub const MANTLE: u32 = 0xff181825;
	pub const MAROON: u32 = 0xffeba0ac;
	pub const MAUVE: u32 = 0xffcba6f7;
	pub const OVERLAY0: u32 = 0xff6c7086;
	pub const OVERLAY1: u32 = 0xff7f849c;
	pub const OVERLAY2: u32 = 0xff9399b2;
	pub const PEACH: u32 = 0xfffab387;
	pub const PINK: u32 = 0xfff5c2e7;
	pub const RED: u32 = 0xfff38ba8;
	pub const ROSEWATER: u32 = 0xfff5e0dc;
	pub const SAPPHIRE: u32 = 0xff74c7ec;
	pub const SKY: u32 = 0xff89dceb;
	pub const SUBTEXT0: u32 = 0xffa6adc8;
	pub const SUBTEXT1: u32 = 0xffbac2de;
	// Base colors
	pub const SURFACE0: u32 = 0xff313244;
	pub const TEAL: u32 = 0xff94e2d5;
	// Text colors
	pub const TEXT: u32 = 0xffcdd6f4;
	// Transparent
	pub const TRANSPARENT: u32 = 0x00000000;
	pub const YELLOW: u32 = 0xfff9e2af;
}

/// Get color based on battery percentage (matching your Lua logic)
pub fn battery_color(percentage: u8, is_charging: bool,) -> u32 {
	if is_charging {
		return Colors::BLUE;
	}

	match percentage {
		95..=100 => Colors::BLUE,
		85..=94 => Colors::SAPPHIRE,
		75..=84 => Colors::SKY,
		65..=74 => Colors::TEAL,
		55..=64 => Colors::GREEN,
		45..=54 => Colors::YELLOW,
		35..=44 => Colors::PEACH,
		25..=34 => Colors::MAROON,
		15..=24 => Colors::RED,
		5..=14 => Colors::RED,
		_ => Colors::RED,
	}
}
