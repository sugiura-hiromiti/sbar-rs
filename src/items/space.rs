use crate::helpers::colors::Colors;
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use serde::Deserialize;
use std::process::Command;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	debug!("🏠 Setting up space items for display {}", display_info.index);

	// Add space items (typically 1-10)
	for i in 1..=10 {
		let space_name = format!("space.{}", i);

		// Add space item
		bar.add("space", &space_name, "left",).await?;

		// Configure space properties
		bar.set(
			&space_name,
			&[
				("icon", &i.to_string(),),
				("icon.color", &format!("0x{:08x}", Colors::TEXT),),
				("background.color", &format!("0x{:08x}", Colors::SURFACE0),),
				("background.border_color", &format!("0x{:08x}", Colors::OVERLAY0),),
				("associated_display", &display_info.index.to_string(),),
			],
		)
		.await?;

		// Subscribe to space events
		bar.subscribe(&space_name, &["space_change", "display_change",],).await?;
	}

	debug!("✅ Space items configured for display {}", display_info.index);
	Ok((),)
}

/// Update space indicators based on current yabai state
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	// Get current spaces from yabai
	let output = Command::new("yabai",).args(["-m", "query", "--spaces",],).output();

	let spaces_info = match output {
		Ok(output,) if output.status.success() => {
			let json_str = String::from_utf8_lossy(&output.stdout,);
			match serde_json::from_str::<Vec<YabaiSpace,>,>(&json_str,) {
				Ok(spaces,) => spaces,
				Err(e,) => {
					warn!("Failed to parse yabai spaces JSON: {}", e);
					return Ok((),); // Don't fail the entire update loop
				},
			}
		},
		Ok(_,) => {
			warn!("yabai command succeeded but returned non-zero status");
			return Ok((),);
		},
		Err(_,) => {
			// yabai not available, skip update
			return Ok((),);
		},
	};

	// Update each space indicator
	for space in spaces_info {
		let space_name = format!("space.{}", space.index);

		let (bg_color, border_color,) = if space.has_focus {
			(Colors::BLUE, Colors::BLUE,)
		} else if space.windows.len() > 0 {
			(Colors::SURFACE0, Colors::GREEN,)
		} else {
			(Colors::SURFACE0, Colors::OVERLAY0,)
		};

		let cmd = format!(
			"--set {} background.color=0x{:08x} background.border_color=0x{:08x}",
			space_name, bg_color, border_color
		);

		if let Err(e,) = bar.message(&cmd,).await {
			error!("Failed to update space {}: {}", space_name, e);
			// Continue with other spaces
		}
	}

	debug!("🏠 Spaces updated");
	Ok((),)
}

/// Update space indicators using centralized state (more efficient)
pub async fn update_with_state(
	bar: &SketchyBar, state: &crate::state::DaemonState,
) -> Result<(),> {
	let spaces = state.spaces.read().await;

	// Update each space indicator based on state
	for space in spaces.values() {
		let space_name = format!("space.{}", space.index);

		let (bg_color, border_color,) = if space.has_focus {
			(Colors::BLUE, Colors::BLUE,)
		} else if !space.windows.is_empty() {
			(Colors::SURFACE0, Colors::GREEN,)
		} else {
			(Colors::SURFACE0, Colors::OVERLAY0,)
		};

		let cmd = format!(
			"--set {} background.color=0x{:08x} background.border_color=0x{:08x}",
			space_name, bg_color, border_color
		);

		if let Err(e,) = bar.message(&cmd,).await {
			error!("Failed to update space {}: {}", space_name, e);
			// Continue with other spaces
		}
	}

	debug!("🏠 Spaces updated from state");
	Ok((),)
}

#[derive(Debug, Deserialize,)]
struct YabaiSpace {
	index:     u32,
	#[serde(rename = "has-focus")]
	has_focus: bool,
	windows:   Vec<u32,>,
}
