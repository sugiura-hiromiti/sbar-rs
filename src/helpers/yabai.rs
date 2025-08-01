use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use tracing::debug;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct DisplayInfo {
	pub index:      u32,
	pub is_builtin: bool,
	pub frame:      DisplayFrame,
}

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct DisplayFrame {
	pub x: f64,
	pub y: f64,
	pub w: f64,
	pub h: f64,
}

/// Get all available displays from yabai
pub async fn get_displays() -> Result<HashMap<String, DisplayInfo,>,> {
	let output = Command::new("yabai",).args(["-m", "query", "--displays",],).output()?;

	if !output.status.success() {
		// Fallback if yabai is not available
		warn!("⚠️  yabai not available, using default display");
		let mut displays = HashMap::new();
		displays.insert(
			"1".to_string(),
			DisplayInfo {
				index:      1,
				is_builtin: true,
				frame:      DisplayFrame { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0, },
			},
		);
		return Ok(displays,);
	}

	let json_str = String::from_utf8(output.stdout,)?;
	let yabai_displays: Vec<YabaiDisplay,> = serde_json::from_str(&json_str,)?;

	let mut displays = HashMap::new();

	for display in yabai_displays {
		let display_info = DisplayInfo {
			index:      display.index,
			is_builtin: display.label.contains("Built-in",) || display.index == 1,
			frame:      DisplayFrame {
				x: display.frame.x,
				y: display.frame.y,
				w: display.frame.w,
				h: display.frame.h,
			},
		};

		displays.insert(display.index.to_string(), display_info,);
	}

	Ok(displays,)
}

/// Query all spaces from yabai
pub async fn query_spaces() -> Result<Vec<crate::state::SpaceInfo,>,> {
	let output = Command::new("yabai",).args(["-m", "query", "--spaces",],).output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("yabai spaces query failed"),);
	}

	let json_str = String::from_utf8(output.stdout,)?;
	let yabai_spaces: Vec<YabaiSpace,> = serde_json::from_str(&json_str,)?;

	let spaces: Vec<crate::state::SpaceInfo,> = yabai_spaces
		.into_iter()
		.map(|s| crate::state::SpaceInfo {
			index:     s.index,
			display:   s.display,
			has_focus: s.has_focus,
			windows:   s.windows,
			label:     s.label,
		},)
		.collect();

	debug!("📊 Queried {} spaces from yabai", spaces.len());
	Ok(spaces,)
}

/// Query all windows from yabai
pub async fn query_windows() -> Result<Vec<crate::state::WindowInfo,>,> {
	let output = Command::new("yabai",).args(["-m", "query", "--windows",],).output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("yabai windows query failed"),);
	}

	let json_str = String::from_utf8(output.stdout,)?;
	let yabai_windows: Vec<YabaiWindow,> = serde_json::from_str(&json_str,)?;

	let windows: Vec<crate::state::WindowInfo,> = yabai_windows
		.into_iter()
		.map(|w| crate::state::WindowInfo {
			id:        w.id,
			app:       w.app,
			title:     w.title,
			space:     w.space,
			display:   w.display,
			has_focus: w.has_focus,
		},)
		.collect();

	debug!("🪟 Queried {} windows from yabai", windows.len());
	Ok(windows,)
}

/// Query focused application
pub async fn query_focused_app() -> Result<String,> {
	// Try yabai first
	if let Ok(output,) =
		Command::new("yabai",).args(["-m", "query", "--windows", "--window",],).output()
	{
		if output.status.success() {
			let json_str = String::from_utf8(output.stdout,)?;
			if let Ok(window,) = serde_json::from_str::<YabaiWindow,>(&json_str,) {
				return Ok(window.app,);
			}
		}
	}

	// Fallback to AppleScript
	let output = Command::new("osascript",)
		.args([
			"-e",
			"tell application \"System Events\" to get name of first application process whose \
			 frontmost is true",
		],)
		.output()?;

	if !output.status.success() {
		return Err(anyhow::anyhow!("AppleScript query failed"),);
	}

	Ok(String::from_utf8(output.stdout,)?.trim().to_string(),)
}

/// Get the builtin display info
pub async fn get_builtin_display() -> Result<Option<DisplayInfo,>,> {
	let displays = get_displays().await?;
	Ok(displays.values().find(|d| d.is_builtin,).cloned(),)
}

/// Get external display indices
pub async fn get_external_displays() -> Result<Vec<DisplayInfo,>,> {
	let displays = get_displays().await?;
	Ok(displays.values().filter(|d| !d.is_builtin,).cloned().collect(),)
}

#[derive(Debug, Deserialize,)]
struct YabaiDisplay {
	index: u32,
	label: String,
	frame: YabaiFrame,
}

#[derive(Debug, Deserialize,)]
struct YabaiFrame {
	x: f64,
	y: f64,
	w: f64,
	h: f64,
}

#[derive(Debug, Deserialize,)]
struct YabaiSpace {
	index:     u32,
	display:   u32,
	#[serde(rename = "has-focus")]
	has_focus: bool,
	windows:   Vec<u32,>,
	label:     String,
}

#[derive(Debug, Deserialize,)]
struct YabaiWindow {
	id:        u32,
	app:       String,
	title:     String,
	space:     u32,
	display:   u32,
	#[serde(rename = "has-focus")]
	has_focus: bool,
}
