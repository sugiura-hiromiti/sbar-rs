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
	debug!("📱 Setting up current app item for display {}", display_info.index);

	// Add current app item
	bar.add("item", "current_app", "left",).await?;

	// Configure current app properties
	bar.set(
		"current_app",
		&[
			("width", "dynamic",),
			("position", "left",),
			("icon", Icons::APP,),
			("icon.color", &format!("0x{:08x}", Colors::MAUVE),),
			("label", "App",),
			("label.color", &format!("0x{:08x}", Colors::MAUVE),),
			("background.border_color", &format!("0x{:08x}", Colors::MAUVE),),
			("associated_display", &display_info.index.to_string(),),
		],
	)
	.await?;

	// Subscribe to app change events
	bar.subscribe("current_app", &["front_app_switched",],).await?;

	debug!("✅ Current app item configured for display {}", display_info.index);
	Ok((),)
}

/// Update current app display with the focused application
pub async fn update(bar: &SketchyBar,) -> Result<(),> {
	// Try to get current app from yabai first
	let app_name = if let Ok(yabai_app,) = get_yabai_focused_app().await {
		yabai_app
	} else {
		// Fallback to AppleScript
		get_applescript_focused_app().await.unwrap_or_else(|_| "Unknown".to_string(),)
	};

	// Update the current app item
	let cmd = format!("--set current_app label={}", app_name);

	if let Err(e,) = bar.message(&cmd,).await {
		error!("Failed to update current app: {}", e);
		return Err(e,);
	}

	debug!("📱 Current app updated: {}", app_name);
	Ok((),)
}

/// Update current app using centralized state (more efficient)
pub async fn update_with_state(
	bar: &SketchyBar, state: &crate::state::DaemonState,
) -> Result<(),> {
	let current_app = state.current_app.read().await;

	if let Some(app_name,) = current_app.as_ref() {
		let cmd = format!("--set current_app label={}", app_name);

		if let Err(e,) = bar.message(&cmd,).await {
			error!("Failed to update current app: {}", e);
			return Err(e,);
		}

		debug!("📱 Current app updated from state: {}", app_name);
	}

	Ok((),)
}

/// Get focused app from yabai
async fn get_yabai_focused_app() -> Result<String,> {
	let output =
		Command::new("yabai",).args(["-m", "query", "--windows", "--window",],).output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("yabai query failed"),);
	}

	let json_str = String::from_utf8(output.stdout,)?;
	let window: YabaiWindow = serde_json::from_str(&json_str,)?;

	Ok(window.app,)
}

/// Get focused app using AppleScript (fallback)
async fn get_applescript_focused_app() -> Result<String,> {
	let output = Command::new("osascript",)
		.args([
			"-e",
			"tell application \"System Events\" to get name of first application process whose \
			 frontmost is true",
		],)
		.output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("AppleScript failed"),);
	}

	let app_name = String::from_utf8(output.stdout,)?.trim().to_string();

	Ok(app_name,)
}

#[derive(Debug, Deserialize,)]
struct YabaiWindow {
	app: String,
}
