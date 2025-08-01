use crate::helpers::colors::Colors;
use crate::helpers::icons::Icons;
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use std::process::Command;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	debug!("⌨️  Setting up keyboard item for display {}", display_info.index);

	// Add keyboard item
	bar.add("item", "keyboard", "right",).await?;

	// Configure keyboard properties
	bar.set(
		"keyboard",
		&[
			("width", "dynamic",),
			("position", "right",),
			("icon", Icons::KEYBOARD,),
			("icon.color", &format!("0x{:08x}", Colors::BLUE),),
			("label", "US",),
			("label.color", &format!("0x{:08x}", Colors::BLUE),),
			("background.border_color", &format!("0x{:08x}", Colors::BLUE),),
		],
	)
	.await?;

	// Only show on builtin display
	if display_info.is_builtin {
		bar.set("keyboard", &[("associated_display", &display_info.index.to_string(),),],).await?;
	}

	debug!("✅ Keyboard item configured for display {}", display_info.index);
	Ok((),)
}

/// Update keyboard display with current input source
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	// Get current input source using defaults command
	let output = Command::new("defaults",)
		.args([
			"read",
			"~/Library/Preferences/com.apple.HIToolbox.plist",
			"AppleSelectedInputSources",
		],)
		.output();

	let input_source = match output {
		Ok(output,) if output.status.success() => {
			let output_str = String::from_utf8_lossy(&output.stdout,);

			// Parse the input source from the plist output
			// This is a simplified parser - in production you might want to use a proper plist
			// parser
			if output_str.contains("U.S.",) || output_str.contains("ABC",) {
				"US"
			} else if output_str.contains("Dvorak",) {
				"DV"
			} else if output_str.contains("Colemak",) {
				"CM"
			} else {
				// Try to extract a short identifier
				"??"
			}
		},
		Ok(_,) => {
			warn!("defaults command succeeded but returned non-zero status");
			"??"
		},
		Err(e,) => {
			error!("Failed to get input source: {}", e);
			return Ok((),); // Don't fail the entire update loop
		},
	};

	// Update the keyboard item
	let cmd = format!("--set keyboard label={}", input_source);

	if let Err(e,) = bar.message(&cmd,).await {
		error!("Failed to update keyboard: {}", e);
		return Err(e,);
	}

	debug!("⌨️  Keyboard updated: {}", input_source);
	Ok((),)
}
