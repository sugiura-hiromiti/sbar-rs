use crate::helpers::colors::Colors;
use crate::helpers::icons::Icons;
use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use serde::Deserialize;
use std::process::Command;
use tracing::debug;
use tracing::error;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	debug!("🪟 Setting up window item for display {}", display_info.index);

	// Add window item
	bar.add("item", "window", "left",).await?;

	// Configure window properties
	bar.set(
		"window",
		&[
			("width", "dynamic",),
			("position", "left",),
			("icon", Icons::WINDOW,),
			("icon.color", &format!("0x{:08x}", Colors::GREEN),),
			("label", "Window",),
			("label.color", &format!("0x{:08x}", Colors::GREEN),),
			("background.border_color", &format!("0x{:08x}", Colors::GREEN),),
			("associated_display", &display_info.index.to_string(),),
		],
	)
	.await?;

	// Subscribe to window events
	bar.subscribe("window", &["window_focus", "window_title",],).await?;

	debug!("✅ Window item configured for display {}", display_info.index);
	Ok((),)
}

/// Update window display with current window title
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	// Try to get current window from yabai first
	let window_title = if let Ok(yabai_title,) = get_yabai_focused_window().await {
		yabai_title
	} else {
		// Fallback to AppleScript
		get_applescript_focused_window().await.unwrap_or_else(|_| "No Window".to_string(),)
	};

	// Truncate long titles
	let display_title = if window_title.len() > 50 {
		format!("{}...", &window_title[..47])
	} else {
		window_title.clone()
	};

	// Update the window item
	let cmd = format!("--set window label=\"{}\"", display_title);

	if let Err(e,) = bar.message(&cmd,).await {
		error!("Failed to update window: {}", e);
		return Err(e,);
	}

	debug!("🪟 Window updated: {}", display_title);
	Ok((),)
}

/// Update window using centralized state (more efficient)
pub async fn update_with_state(
	bar: &SketchyBar, state: &crate::state::DaemonState,
) -> Result<(),> {
	let windows = state.windows.read().await;

	// Find the focused window
	if let Some(focused_window,) = windows.values().find(|w| w.has_focus,) {
		let display_title = if focused_window.title.len() > 50 {
			format!("{}...", &focused_window.title[..47])
		} else {
			focused_window.title.clone()
		};

		let cmd = format!("--set window label=\"{}\"", display_title);

		if let Err(e,) = bar.message(&cmd,).await {
			error!("Failed to update window: {}", e);
			return Err(e,);
		}

		debug!("🪟 Window updated from state: {}", display_title);
	} else {
		// No focused window
		let cmd = "--set window label=\"No Window\"";
		if let Err(e,) = bar.message(cmd,).await {
			error!("Failed to update window: {}", e);
			return Err(e,);
		}
	}

	Ok((),)
}

/// Get focused window title from yabai
async fn get_yabai_focused_window() -> Result<String,> {
	let output =
		Command::new("yabai",).args(["-m", "query", "--windows", "--window",],).output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("yabai query failed"),);
	}

	let json_str = String::from_utf8(output.stdout,)?;
	let window: YabaiWindow = serde_json::from_str(&json_str,)?;

	Ok(window.title,)
}

/// Get focused window title using AppleScript (fallback)
async fn get_applescript_focused_window() -> Result<String,> {
	let output = Command::new("osascript",)
		.args([
			"-e",
			"tell application \"System Events\" to get name of window 1 of (first application \
			 process whose frontmost is true)",
		],)
		.output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("AppleScript failed"),);
	}

	let window_title = String::from_utf8(output.stdout,)?.trim().to_string();

	Ok(window_title,)
}

#[derive(Debug, Deserialize,)]
struct YabaiWindow {
	title: String,
}
