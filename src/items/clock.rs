use crate::helpers::colors::Colors;
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use chrono::Local;
use tracing::debug;
use tracing::error;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	debug!("🕐 Setting up clock item for display {}", display_info.index);

	// Add clock item
	bar.add("item", "clock", "right",).await?;

// Configure clock properties
	bar.set(
		"clock",
		&[
			("update_freq", "1",),
			("width", "dynamic",),
			("position", "right",),
			("label.color", &format!("0x{:08x}", Colors::FLAMINGO),),
			("background.border_color", &format!("0x{:08x}", Colors::FLAMINGO),),
		],
	)
	.await?;

	// Only show on builtin display
	if display_info.is_builtin {
		bar.set("clock", &[("associated_display", &display_info.index.to_string(),),],).await?;
	}

	// Subscribe to events
	bar.subscribe("clock", &["system_woke", "routine",],).await?;

	debug!("✅ Clock item configured for display {}", display_info.index);
	Ok((),)
}

/// Update clock display with current time
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	let now = Local::now();
	let time_str = now.format("%y%m%d %H%M %a",).to_string();

	let cmd = format!("--set clock label={}", time_str);

	if let Err(e,) = bar.message(&cmd,).await {
		error!("Failed to update clock: {}", e);
		return Err(e,);
	}

	debug!("🕐 Clock updated: {}", time_str);
	Ok((),)
}
