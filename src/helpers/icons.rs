/// Icon constants using Nerd Font symbols (matching your Lua config)
pub struct Icons;

impl Icons {
	pub const APP: &'static str = "\u{f013}";
	// Battery icons
	pub const BATTERY_10: &'static str = "\u{f007a}";
	pub const BATTERY_100: &'static str = "\u{f0079}";
	pub const BATTERY_20: &'static str = "\u{f007b}";
	pub const BATTERY_30: &'static str = "\u{f007c}";
	pub const BATTERY_40: &'static str = "\u{f007d}";
	pub const BATTERY_50: &'static str = "\u{f007e}";
	pub const BATTERY_60: &'static str = "\u{f007f}";
	pub const BATTERY_70: &'static str = "\u{f0080}";
	pub const BATTERY_80: &'static str = "\u{f0081}";
	pub const BATTERY_90: &'static str = "\u{f0082}";
	pub const BATTERY_CHARGING: &'static str = "\u{f0084}";
	pub const CHECK: &'static str = "\u{eab2}";
	// Additional common icons
	pub const CLOCK: &'static str = "\u{f017}";
	// General icons
	pub const ERROR: &'static str = "\u{ea87}";
	pub const KEYBOARD: &'static str = "\u{f11c}";
	pub const SPACE: &'static str = "\u{f0c8}";
	pub const WINDOW: &'static str = "\u{f2d0}";
}

/// Get battery icon based on percentage
pub fn battery_icon(percentage: u8, is_charging: bool,) -> &'static str {
	if is_charging {
		return Icons::BATTERY_CHARGING;
	}

	match percentage {
		95..=100 => Icons::BATTERY_100,
		85..=94 => Icons::BATTERY_90,
		75..=84 => Icons::BATTERY_80,
		65..=74 => Icons::BATTERY_70,
		55..=64 => Icons::BATTERY_60,
		45..=54 => Icons::BATTERY_50,
		35..=44 => Icons::BATTERY_40,
		25..=34 => Icons::BATTERY_30,
		15..=24 => Icons::BATTERY_20,
		5..=14 => Icons::BATTERY_10,
		_ => Icons::ERROR,
	}
}
