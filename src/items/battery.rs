use crate::helpers::colors::Colors;
use crate::helpers::colors::{self};
use crate::helpers::icons::Icons;
use crate::helpers::icons::{self};
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use regex::Regex;
use std::process::Command;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	debug!("🔋 Setting up battery item for display {}", display_info.index);

	// Add battery item
	bar.add("item", "battery", "right",).await?;

	// Configure battery properties
	bar.set(
		"battery",
		&[
			("width", "dynamic",),
			("position", "right",),
			("associated_display", &display_info.index.to_string(),),
		],
	)
	.await?;

	// Subscribe to events
	bar.subscribe("battery", &["routine", "power_source_change", "system_woke",],).await?;

	debug!("✅ Battery item configured for display {}", display_info.index);
	Ok((),)
}

/// Update battery display with current status
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	// Get battery information using pmset
	let output = Command::new("pmset",)
		.args(["-g", "batt",],)
		.output()
		.map_err(|e| anyhow::anyhow!("Failed to run pmset: {}", e),)?;

	if !output.status.success() {
		warn!("pmset command failed with status: {}", output.status);
		return Ok((),); // Don't fail the entire update loop
	}

	let batt_info = String::from_utf8(output.stdout,)
		.map_err(|e| anyhow::anyhow!("Failed to parse pmset output: {}", e),)?;

	let mut icon = Icons::ERROR;
	let mut label = "?".to_string();
	let mut color = Colors::YELLOW;

	// Parse battery percentage
	let re = Regex::new(r"(\d+)%",)?;
	if let Some(captures,) = re.captures(&batt_info,) {
		if let Some(charge_match,) = captures.get(1,) {
			let charge: u8 = charge_match.as_str().parse().unwrap_or(0,);
			label = if charge < 10 { format!("0{}", charge) } else { charge.to_string() };

			// Check if charging
			let is_charging = batt_info.contains("AC Power",);

			// Get appropriate icon and color
			icon = icons::battery_icon(charge, is_charging,);
			color = colors::battery_color(charge, is_charging,);
		}
	} else {
		warn!("Could not parse battery percentage from pmset output");
		return Ok((),); // Don't fail the entire update loop
	}

	// Update the battery item
	let cmd = format!(
		"--set battery icon={} icon.color=0x{:08x} icon.padding_left=10 label={} \
		 label.color=0x{:08x} label.padding_right=10",
		icon, color, label, color
	);

	if let Err(e,) = bar.message(&cmd,).await {
		error!("Failed to update battery: {}", e);
		return Err(e,);
	}

	debug!(
		"🔋 Battery updated: {}% ({})",
		label,
		if batt_info.contains("AC Power") { "charging" } else { "discharging" }
	);
	Ok((),)
}
