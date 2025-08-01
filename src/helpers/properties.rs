use super::colors::Colors;
use super::yabai::DisplayInfo;

/// Configuration properties for different display types
pub struct Properties;

impl Properties {
	/// Get bar properties based on display type
	pub fn bar_properties(display_info: &DisplayInfo,) -> BarProperties {
		if display_info.is_builtin {
			BarProperties {
				position:           "top".to_string(),
				height:             56,
				sticky:             true,
				shadow:             false,
				font_smoothing:     false,
				show_in_fullscreen: true,
				margin:             0,
				color:              Colors::TRANSPARENT,
				y_offset:           8,
				padding_left:       2,
				padding_right:      2,
				display:            display_info.index,
				topmost:            true,
			}
		} else {
			BarProperties {
				position:           "bottom".to_string(),
				height:             26,
				sticky:             true,
				shadow:             false,
				font_smoothing:     false,
				show_in_fullscreen: true,
				margin:             0,
				color:              Colors::TRANSPARENT,
				y_offset:           0,
				padding_left:       2,
				padding_right:      2,
				display:            display_info.index,
				topmost:            true,
			}
		}
	}

	/// Get default item properties based on display type
	pub fn default_properties(display_info: &DisplayInfo,) -> DefaultProperties {
		let (padding, background_height, corner_radius, font_size,) =
			if display_info.is_builtin { (4, 40, 10, 16,) } else { (2, 20, 5, 14,) };

		let label_padding = if display_info.is_builtin { 10 } else { 4 };

		DefaultProperties {
			update_freq:        "when_shown".to_string(),
			position:           "left".to_string(),
			ignore_association: false,
			y_offset:           0,
			padding_left:       padding,
			padding_right:      padding,
			width:              "dynamic".to_string(),
			scroll_texts:       true,
			blur_radius:        25,
			align:              "center".to_string(),
			background:         BackgroundProperties {
				drawing: true,
				color: Colors::SURFACE0,
				border_color: 0xffffffff,
				border_width: 1,
				height: background_height,
				corner_radius,
			},
			icon:               FontProperties {
				family: "MesloLGL Nerd Font".to_string(),
				style:  "Regular".to_string(),
				size:   font_size,
			},
			label:              LabelProperties {
				font:          FontProperties {
					family: "MesloLGL Nerd Font".to_string(),
					style:  "Regular".to_string(),
					size:   font_size,
				},
				padding_left:  label_padding,
				padding_right: label_padding,
			},
		}
	}
}

#[derive(Debug, Clone,)]
pub struct BarProperties {
	pub position:           String,
	pub height:             u32,
	pub sticky:             bool,
	pub shadow:             bool,
	pub font_smoothing:     bool,
	pub show_in_fullscreen: bool,
	pub margin:             u32,
	pub color:              u32,
	pub y_offset:           i32,
	pub padding_left:       u32,
	pub padding_right:      u32,
	pub display:            u32,
	pub topmost:            bool,
}

#[derive(Debug, Clone,)]
pub struct DefaultProperties {
	pub update_freq:        String,
	pub position:           String,
	pub ignore_association: bool,
	pub y_offset:           i32,
	pub padding_left:       u32,
	pub padding_right:      u32,
	pub width:              String,
	pub scroll_texts:       bool,
	pub blur_radius:        u32,
	pub align:              String,
	pub background:         BackgroundProperties,
	pub icon:               FontProperties,
	pub label:              LabelProperties,
}

#[derive(Debug, Clone,)]
pub struct BackgroundProperties {
	pub drawing:       bool,
	pub color:         u32,
	pub border_color:  u32,
	pub border_width:  u32,
	pub height:        u32,
	pub corner_radius: u32,
}

#[derive(Debug, Clone,)]
pub struct FontProperties {
	pub family: String,
	pub style:  String,
	pub size:   u32,
}

#[derive(Debug, Clone,)]
pub struct LabelProperties {
	pub font:          FontProperties,
	pub padding_left:  u32,
	pub padding_right: u32,
}
