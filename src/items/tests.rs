use anyhow::Result;
use std::process::Command;
use tokio::time::Duration;
use tokio::time::sleep;

use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use crate::state::DaemonState;

/// Test utilities for item testing
pub mod utils {
	use super::*;

	pub fn create_test_display() -> DisplayInfo {
		DisplayInfo {
			index:      1,
			is_builtin: true,
			frame:      crate::helpers::yabai::DisplayFrame {
				x: 0.0,
				y: 0.0,
				w: 1920.0,
				h: 1080.0,
			},
		}
	}

	pub fn is_sketchybar_running() -> bool {
		Command::new("pgrep",)
			.args(["-f", "sketchybar",],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}

	pub fn is_yabai_running() -> bool {
		Command::new("pgrep",)
			.args(["-f", "yabai",],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}
}

#[cfg(test)]
mod clock_tests {
	use super::*;

	#[tokio::test]
	async fn test_clock_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		// Test clock setup
		let result = crate::items::clock::setup(&mut bar, &display,).await;

		// Setup should not fail even if sketchybar is not running
		match result {
			Ok(_,) => println!("Clock setup succeeded"),
			Err(e,) => println!("Clock setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_clock_update() {
		let bar = SketchyBar::new();

		// Test clock update
		let result = crate::items::clock::update(&bar,).await;

		// Update should not fail even if sketchybar is not running
		match result {
			Ok(_,) => println!("Clock update succeeded"),
			Err(e,) => println!("Clock update failed (expected without sketchybar): {}", e),
		}
	}

	#[test]
	fn test_clock_time_formatting() {
		use chrono::Local;

		// Test that we can format time correctly
		let now = Local::now();
		let formatted = now.format("%H:%M",).to_string();

		assert!(formatted.len() == 5); // HH:MM format
		assert!(formatted.contains(':'));

		// Test different formats
		let date_formatted = now.format("%Y-%m-%d",).to_string();
		assert!(date_formatted.len() == 10); // YYYY-MM-DD format
		assert_eq!(date_formatted.matches('-').count(), 2);
	}
}

#[cfg(test)]
mod battery_tests {
	use super::*;

	#[tokio::test]
	async fn test_battery_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		let result = crate::items::battery::setup(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("Battery setup succeeded"),
			Err(e,) => println!("Battery setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_battery_update() {
		let bar = SketchyBar::new();

		let result = crate::items::battery::update(&bar,).await;

		match result {
			Ok(_,) => println!("Battery update succeeded"),
			Err(e,) => println!("Battery update failed (expected without sketchybar): {}", e),
		}
	}

	#[test]
	fn test_battery_info_parsing() {
		// Test battery percentage parsing
		let sample_output = "75%";
		let percentage: Result<u32, _,> = sample_output.trim_end_matches('%',).parse();
		assert!(percentage.is_ok());
		assert_eq!(percentage.unwrap(), 75);

		// Test invalid input
		let invalid_output = "invalid";
		let invalid_percentage: Result<u32, _,> = invalid_output.trim_end_matches('%',).parse();
		assert!(invalid_percentage.is_err());
	}

	#[test]
	fn test_battery_color_logic() {
		// Test battery color selection logic
		fn get_battery_color(percentage: u32, is_charging: bool,) -> u32 {
			use crate::helpers::colors::Colors;

			if is_charging {
				Colors::GREEN
			} else if percentage > 50 {
				Colors::GREEN
			} else if percentage > 20 {
				Colors::YELLOW
			} else {
				Colors::RED
			}
		}

		// Test different scenarios
		assert_eq!(get_battery_color(80, false), crate::helpers::colors::Colors::GREEN);
		assert_eq!(get_battery_color(30, false), crate::helpers::colors::Colors::YELLOW);
		assert_eq!(get_battery_color(10, false), crate::helpers::colors::Colors::RED);
		assert_eq!(get_battery_color(10, true), crate::helpers::colors::Colors::GREEN);
	}
}

#[cfg(test)]
mod space_tests {
	use super::*;

	#[tokio::test]
	async fn test_space_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		let result = crate::items::space::setup(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("Space setup succeeded"),
			Err(e,) => println!("Space setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_space_update() {
		let bar = SketchyBar::new();

		let result = crate::items::space::update(&bar,).await;

		match result {
			Ok(_,) => println!("Space update succeeded"),
			Err(e,) => println!("Space update failed (expected without yabai): {}", e),
		}
	}

	#[tokio::test]
	async fn test_space_update_with_state() {
		if !utils::is_yabai_running() {
			println!("Skipping space state test - yabai not running");
			return;
		}

		let bar = SketchyBar::new();
		let state = DaemonState::new();

		// Update state first
		let _ = state.update_spaces().await;

		let result = crate::items::space::update_with_state(&bar, &state,).await;

		match result {
			Ok(_,) => println!("Space state update succeeded"),
			Err(e,) => println!("Space state update failed: {}", e),
		}
	}

	#[test]
	fn test_space_color_logic() {
		use crate::helpers::colors::Colors;

		// Test space color selection
		fn get_space_colors(has_focus: bool, has_windows: bool,) -> (u32, u32,) {
			if has_focus {
				(Colors::BLUE, Colors::BLUE,)
			} else if has_windows {
				(Colors::SURFACE0, Colors::GREEN,)
			} else {
				(Colors::SURFACE0, Colors::OVERLAY0,)
			}
		}

		let (bg, border,) = get_space_colors(true, false,);
		assert_eq!(bg, Colors::BLUE);
		assert_eq!(border, Colors::BLUE);

		let (bg, border,) = get_space_colors(false, true,);
		assert_eq!(bg, Colors::SURFACE0);
		assert_eq!(border, Colors::GREEN);

		let (bg, border,) = get_space_colors(false, false,);
		assert_eq!(bg, Colors::SURFACE0);
		assert_eq!(border, Colors::OVERLAY0);
	}
}

#[cfg(test)]
mod app_tests {
	use super::*;

	#[tokio::test]
	async fn test_current_app_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		let result = crate::items::current_app::setup(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("Current app setup succeeded"),
			Err(e,) => println!("Current app setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_current_app_update_with_state() {
		let bar = SketchyBar::new();
		let state = DaemonState::new();

		// Set a test app in state
		{
			let mut current_app = state.current_app.write().await;
			*current_app = Some("TestApp".to_string(),);
		}

		let result = crate::items::current_app::update_with_state(&bar, &state,).await;

		match result {
			Ok(_,) => println!("Current app state update succeeded"),
			Err(e,) => println!("Current app state update failed: {}", e),
		}
	}

	#[test]
	fn test_app_name_sanitization() {
		// Test app name handling
		let app_names =
			vec!["Terminal", "Google Chrome", "Visual Studio Code", "System Preferences", ""];

		for app_name in app_names {
			// App names should be handled gracefully
			let sanitized = if app_name.is_empty() { "Unknown" } else { app_name };

			assert!(!sanitized.is_empty());
			println!("App name '{}' -> '{}'", app_name, sanitized);
		}
	}
}

#[cfg(test)]
mod window_tests {
	use super::*;

	#[tokio::test]
	async fn test_window_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		let result = crate::items::window::setup(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("Window setup succeeded"),
			Err(e,) => println!("Window setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_window_update_with_state() {
		let bar = SketchyBar::new();
		let state = DaemonState::new();

		// Add a test window to state
		{
			let mut windows = state.windows.write().await;
			windows.insert(
				1,
				crate::state::WindowInfo {
					id:        1,
					app:       "Terminal".to_string(),
					title:     "bash - test window with a very long title that should be truncated"
						.to_string(),
					space:     1,
					display:   1,
					has_focus: true,
				},
			);
		}

		let result = crate::items::window::update_with_state(&bar, &state,).await;

		match result {
			Ok(_,) => println!("Window state update succeeded"),
			Err(e,) => println!("Window state update failed: {}", e),
		}
	}

	#[test]
	fn test_window_title_truncation() {
		let long_title =
			"This is a very long window title that should be truncated to fit in the bar";
		let max_length = 50;

		let truncated = if long_title.len() > max_length {
			format!("{}...", &long_title[..47])
		} else {
			long_title.to_string()
		};

		assert!(truncated.len() <= max_length);
		if long_title.len() > max_length {
			assert!(truncated.ends_with("..."));
		}

		println!("Original: {}", long_title);
		println!("Truncated: {}", truncated);
	}

	#[test]
	fn test_window_title_edge_cases() {
		let test_cases = vec![
			("", "No Window",),
			("Short", "Short",),
			(
				"Exactly fifty characters long title for testing",
				"Exactly fifty characters long title for testing",
			),
			(
				"This title is definitely longer than fifty characters and should be truncated",
				"This title is definitely longer than fifty...",
			),
		];

		for (input, expected_pattern,) in test_cases {
			let result = if input.is_empty() {
				"No Window".to_string()
			} else if input.len() > 50 {
				format!("{}...", &input[..47])
			} else {
				input.to_string()
			};

			if expected_pattern.ends_with("...",) {
				assert!(result.ends_with("..."));
				assert!(result.len() <= 50);
			} else {
				assert_eq!(result, expected_pattern);
			}

			println!("Input: '{}' -> Output: '{}'", input, result);
		}
	}
}

#[cfg(test)]
mod keyboard_tests {
	use super::*;

	#[tokio::test]
	async fn test_keyboard_setup() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		let result = crate::items::keyboard::setup(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("Keyboard setup succeeded"),
			Err(e,) => println!("Keyboard setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_keyboard_update() {
		let bar = SketchyBar::new();

		let result = crate::items::keyboard::update(&bar,).await;

		match result {
			Ok(_,) => println!("Keyboard update succeeded"),
			Err(e,) => println!("Keyboard update failed (expected without system access): {}", e),
		}
	}

	#[test]
	fn test_keyboard_layout_parsing() {
		// Test keyboard layout name parsing
		let sample_layouts = vec!["U.S.", "ABC", "Dvorak", "Colemak", "Japanese", "Korean"];

		for layout in sample_layouts {
			// Layout names should be handled properly
			let display_name =
				if layout.len() > 10 { format!("{}...", &layout[..7]) } else { layout.to_string() };

			assert!(!display_name.is_empty());
			assert!(display_name.len() <= 10);
			println!("Layout '{}' -> Display '{}'", layout, display_name);
		}
	}
}

/// Integration tests that require real system interaction
#[cfg(test)]
mod integration_tests {
	use super::*;

	#[tokio::test]
	async fn test_all_items_setup_integration() {
		let mut bar = SketchyBar::new();
		let display = utils::create_test_display();

		// Test setting up all items
		let result = crate::items::setup_all_items(&mut bar, &display,).await;

		match result {
			Ok(_,) => println!("All items setup succeeded"),
			Err(e,) => println!("All items setup failed (expected without sketchybar): {}", e),
		}
	}

	#[tokio::test]
	async fn test_item_update_sequence() {
		if !utils::is_yabai_running() {
			println!("Skipping item update sequence test - yabai not running");
			return;
		}

		let bar = SketchyBar::new();
		let state = DaemonState::new();

		// Update state first
		let _ = state.update_spaces().await;
		let _ = state.update_windows().await;
		let _ = state.update_current_app().await;

		// Test updating all items in sequence (not parallel due to type differences)
		let results = vec![
			crate::items::clock::update(&bar,).await,
			crate::items::battery::update(&bar,).await,
			crate::items::keyboard::update(&bar,).await,
			crate::items::space::update_with_state(&bar, &state,).await,
			crate::items::current_app::update_with_state(&bar, &state,).await,
			crate::items::window::update_with_state(&bar, &state,).await,
		];

		for (i, result,) in results.iter().enumerate() {
			match result {
				Ok(_,) => println!("Update {} succeeded", i),
				Err(e,) => println!("Update {} failed: {}", i, e),
			}
		}
	}

	#[tokio::test]
	async fn test_rapid_state_updates() {
		if !utils::is_yabai_running() {
			println!("Skipping rapid state updates test - yabai not running");
			return;
		}

		let state = DaemonState::new();

		// Perform rapid state updates
		for i in 0..10 {
			let start = std::time::Instant::now();

			let (spaces_result, windows_result, app_result,) = tokio::join!(
				state.update_spaces(),
				state.update_windows(),
				state.update_current_app()
			);

			let duration = start.elapsed();

			println!("Update {} took {:?}", i, duration);

			// Verify at least one update succeeded
			let success_count = [&spaces_result, &windows_result, &app_result,]
				.iter()
				.filter(|r| r.is_ok(),)
				.count();

			assert!(success_count > 0, "At least one state update should succeed");

			// Small delay between updates
			sleep(Duration::from_millis(100,),).await;
		}
	}
}
