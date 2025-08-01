use crate::helpers::properties::Properties;
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use tracing::debug;
use tracing::info;

/// Setup and configure a SketchyBar instance
pub async fn setup_bar(
	bar: &mut SketchyBar,
	bar_name: &str,
	display_info: &DisplayInfo,
) -> Result<(),> {
	info!("🔧 Configuring bar '{}' for display {}", bar_name, display_info.index);

	// Get properties for this display type
	let bar_props = Properties::bar_properties(display_info,);
	let default_props = Properties::default_properties(display_info,);

	debug!("Bar properties: {:?}", bar_props);
	debug!("Default properties: {:?}", default_props);

	// Configure bar properties
	bar.bar(&[
		("position", &bar_props.position,),
		("height", &bar_props.height.to_string(),),
		("sticky", &bar_props.sticky.to_string(),),
		("shadow", &bar_props.shadow.to_string(),),
		("font_smoothing", &bar_props.font_smoothing.to_string(),),
		("margin", &bar_props.margin.to_string(),),
		("color", &format!("0x{:08x}", bar_props.color),),
		("y_offset", &bar_props.y_offset.to_string(),),
		("padding_left", &bar_props.padding_left.to_string(),),
		("padding_right", &bar_props.padding_right.to_string(),),
		("display", &bar_props.display.to_string(),),
		("topmost", &bar_props.topmost.to_string(),),
	],)
		.await?;

	// Set default properties for items
	bar.default(&[
		("update_freq", &default_props.update_freq,),
		("position", &default_props.position,),
		("y_offset", &default_props.y_offset.to_string(),),
		("padding_left", &default_props.padding_left.to_string(),),
		("padding_right", &default_props.padding_right.to_string(),),
		("width", &default_props.width,),
		("scroll_texts", &default_props.scroll_texts.to_string(),),
		("blur_radius", &default_props.blur_radius.to_string(),),
		("align", &default_props.align,),
		("background.drawing", &default_props.background.drawing.to_string(),),
		("background.color", &format!("0x{:08x}", default_props.background.color),),
		("background.border_color", &format!("0x{:08x}", default_props.background.border_color),),
		("background.border_width", &default_props.background.border_width.to_string(),),
		("background.height", &default_props.background.height.to_string(),),
		("background.corner_radius", &default_props.background.corner_radius.to_string(),),
		("icon.font.family", &default_props.icon.family,),
		("icon.font.style", &default_props.icon.style,),
		("icon.font.size", &default_props.icon.size.to_string(),),
		("label.font.family", &default_props.label.font.family,),
		("label.font.style", &default_props.label.font.style,),
		("label.font.size", &default_props.label.font.size.to_string(),),
		("label.padding_left", &default_props.label.padding_left.to_string(),),
		("label.padding_right", &default_props.label.padding_right.to_string(),),
	],)
		.await?;

	info!("✅ Bar '{}' configuration complete", bar_name);
	Ok((),)
}
