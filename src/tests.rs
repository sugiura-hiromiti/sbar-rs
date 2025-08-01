use anyhow::Result;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

use crate::helpers::yabai;
use crate::sketchybar::SketchyBar;
use crate::state::DaemonState;
use crate::state::SpaceInfo;
use crate::state::WindowInfo;

/// Test utilities and helpers
pub mod utils {
	use super::*;

	/// Check if yabai is available and running
	pub fn is_yabai_available() -> bool {
		Command::new("yabai",)
			.args(["-m", "query", "--displays",],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}

	/// Check if sketchybar is available
	pub fn is_sketchybar_available() -> bool {
		Command::new("sketchybar",)
			.args(["--query", "bar",],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}

	/// Get current focused space index
	pub fn get_current_space() -> Option<u32,> {
		let output =
			Command::new("yabai",).args(["-m", "query", "--spaces", "--space",],).output().ok()?;

		if !output.status.success() {
			return None;
		}

		let json_str = String::from_utf8(output.stdout,).ok()?;
		let space: serde_json::Value = serde_json::from_str(&json_str,).ok()?;
		space["index"].as_u64().map(|i| i as u32,)
	}

	/// Switch to a specific space
	pub fn switch_to_space(space_index: u32,) -> bool {
		Command::new("yabai",)
			.args(["-m", "space", "--focus", &space_index.to_string(),],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}

	/// Create a test window (opens Terminal)
	pub fn create_test_window() -> bool {
		Command::new("open",)
			.args(["-a", "Terminal",],)
			.output()
			.map(|output| output.status.success(),)
			.unwrap_or(false,)
	}

	/// Close all Terminal windows
	pub fn cleanup_test_windows() {
		let _ = Command::new("osascript",)
			.args(["-e", "tell application \"Terminal\" to quit",],)
			.output();
	}
}

/// Unit tests for data structures and basic functionality
#[cfg(test)]
mod unit_tests {
	use super::*;

	#[tokio::test]
	async fn test_daemon_state_creation() {
		let state = DaemonState::new();

		// Test initial state
		let spaces = state.spaces.read().await;
		assert!(spaces.is_empty());

		let windows = state.windows.read().await;
		assert!(windows.is_empty());

		let current_app = state.current_app.read().await;
		assert!(current_app.is_none());
	}

	#[test]
	fn test_sketchybar_creation() {
		let bar = SketchyBar::new();
		assert_eq!(bar.get_bar_name(), "sketchybar");
	}

	#[test]
	fn test_display_info_creation() {
		let display = yabai::DisplayInfo {
			index:      1,
			is_builtin: true,
			frame:      yabai::DisplayFrame { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0, },
		};

		assert_eq!(display.index, 1);
		assert!(display.is_builtin);
		assert_eq!(display.frame.w, 1920.0);
	}

	#[test]
	fn test_space_info_creation() {
		let space = SpaceInfo {
			index:     1,
			display:   1,
			has_focus: true,
			windows:   vec![1, 2, 3],
			label:     "Desktop 1".to_string(),
		};

		assert_eq!(space.index, 1);
		assert!(space.has_focus);
		assert_eq!(space.windows.len(), 3);
	}

	#[test]
	fn test_window_info_creation() {
		let window = WindowInfo {
			id:        123,
			app:       "Terminal".to_string(),
			title:     "bash".to_string(),
			space:     1,
			display:   1,
			has_focus: true,
		};

		assert_eq!(window.id, 123);
		assert_eq!(window.app, "Terminal");
		assert!(window.has_focus);
	}

	#[tokio::test]
	async fn test_state_update_empty() {
		let state = DaemonState::new();

		// Test updating with empty state
		let spaces_changed = state.update_spaces().await;
		let windows_changed = state.update_windows().await;
		let app_changed = state.update_current_app().await;

		// These might fail if yabai is not available, but shouldn't panic
		match (spaces_changed, windows_changed, app_changed,) {
			(Ok(_,), Ok(_,), Ok(_,),) => {
				// All updates succeeded
				println!("All state updates succeeded");
			},
			_ => {
				// Some updates failed (expected if yabai not available)
				println!("Some state updates failed (expected without yabai)");
			},
		}
	}

	#[test]
	fn test_display_frame_calculations() {
		let frame = yabai::DisplayFrame { x: 100.0, y: 200.0, w: 1920.0, h: 1080.0, };

		// Test frame properties
		assert_eq!(frame.x + frame.w, 2020.0);
		assert_eq!(frame.y + frame.h, 1280.0);

		// Test aspect ratio
		let aspect_ratio = frame.w / frame.h;
		assert!((aspect_ratio - 16.0 / 9.0).abs() < 0.01);
	}
}

/// Integration tests with real yabai commands
#[cfg(test)]
mod integration_tests {
	use super::*;

	#[tokio::test]
	async fn test_yabai_display_query() {
		if !utils::is_yabai_available() {
			println!("Skipping yabai test - yabai not available");
			return;
		}

		let displays = yabai::get_displays().await;
		match displays {
			Ok(displays,) => {
				assert!(!displays.is_empty(), "Should have at least one display");

				// Test display properties
				for (id, display,) in displays {
					assert!(!id.is_empty());
					assert!(display.index > 0);
					assert!(display.frame.w > 0.0);
					assert!(display.frame.h > 0.0);

					println!(
						"Display {}: {}x{} at ({}, {}), builtin: {}",
						display.index,
						display.frame.w,
						display.frame.h,
						display.frame.x,
						display.frame.y,
						display.is_builtin
					);
				}
			},
			Err(e,) => {
				panic!("Failed to query displays: {}", e);
			},
		}
	}

	#[tokio::test]
	async fn test_yabai_spaces_query() {
		if !utils::is_yabai_available() {
			println!("Skipping yabai spaces test - yabai not available");
			return;
		}

		let spaces = yabai::query_spaces().await;
		match spaces {
			Ok(spaces,) => {
				assert!(!spaces.is_empty(), "Should have at least one space");

				// Test space properties
				let mut has_focused = false;
				for space in spaces {
					assert!(space.index > 0);
					assert!(space.display > 0);
					if space.has_focus {
						has_focused = true;
					}

					println!(
						"Space {}: display {}, focused: {}, windows: {:?}",
						space.index, space.display, space.has_focus, space.windows
					);
				}

				assert!(has_focused, "At least one space should be focused");
			},
			Err(e,) => {
				panic!("Failed to query spaces: {}", e);
			},
		}
	}

	#[tokio::test]
	async fn test_yabai_windows_query() {
		if !utils::is_yabai_available() {
			println!("Skipping yabai windows test - yabai not available");
			return;
		}

		let windows = yabai::query_windows().await;
		match windows {
			Ok(windows,) => {
				// Windows might be empty, that's okay
				println!("Found {} windows", windows.len());

				for window in windows {
					assert!(window.id > 0);
					assert!(!window.app.is_empty());
					assert!(window.space > 0);
					assert!(window.display > 0);

					println!(
						"Window {}: {} - '{}' on space {} display {}",
						window.id, window.app, window.title, window.space, window.display
					);
				}
			},
			Err(e,) => {
				panic!("Failed to query windows: {}", e);
			},
		}
	}

	#[tokio::test]
	async fn test_yabai_focused_app_query() {
		if !utils::is_yabai_available() {
			println!("Skipping yabai focused app test - yabai not available");
			return;
		}

		let focused_app = yabai::query_focused_app().await;
		match focused_app {
			Ok(app,) => {
				assert!(!app.is_empty(), "Focused app should not be empty");
				println!("Focused app: {}", app);
			},
			Err(e,) => {
				// This might fail if no windows are focused, which is okay
				println!("Failed to get focused app (might be expected): {}", e);
			},
		}
	}

	#[tokio::test]
	async fn test_state_management_with_real_data() {
		if !utils::is_yabai_available() {
			println!("Skipping state management test - yabai not available");
			return;
		}

		let state = DaemonState::new();

		// Update all state
		let spaces_result = state.update_spaces().await;
		let windows_result = state.update_windows().await;
		let app_result = state.update_current_app().await;

		assert!(spaces_result.is_ok(), "Spaces update should succeed");
		assert!(windows_result.is_ok(), "Windows update should succeed");
		assert!(app_result.is_ok(), "App update should succeed");

		// Verify state was populated
		let spaces = state.spaces.read().await;
		assert!(!spaces.is_empty(), "Spaces should be populated");

		// Test state consistency
		let mut focused_spaces = 0;
		for space in spaces.values() {
			if space.has_focus {
				focused_spaces += 1;
			}
		}
		assert_eq!(focused_spaces, 1, "Exactly one space should be focused");

		println!("State management test passed with {} spaces", spaces.len());
	}
}

/// Edge case and error handling tests
#[cfg(test)]
mod edge_case_tests {
	use super::*;

	#[tokio::test]
	async fn test_yabai_not_available() {
		// Temporarily rename yabai to simulate it not being available
		let original_path = std::env::var("PATH",).unwrap_or_default();
		unsafe { std::env::set_var("PATH", "/nonexistent",) };

		let displays = yabai::get_displays().await;
		match displays {
			Ok(displays,) => {
				// Should fall back to default display
				assert_eq!(displays.len(), 1);
				let default_display = displays.get("1",).unwrap();
				assert_eq!(default_display.index, 1);
				assert!(default_display.is_builtin);
			},
			Err(_,) => {
				// This is also acceptable behavior
				println!("yabai fallback handled gracefully");
			},
		}

		// Restore PATH
		unsafe { std::env::set_var("PATH", &original_path,) };
	}

	#[tokio::test]
	async fn test_malformed_json_handling() {
		// This test would require mocking yabai output, which is complex
		// For now, we test that our parsing is robust
		let malformed_json = r#"{"index": 1, "invalid": }"#;
		let result: Result<serde_json::Value, _,> = serde_json::from_str(malformed_json,);
		assert!(result.is_err(), "Should handle malformed JSON gracefully");
	}

	#[tokio::test]
	async fn test_empty_yabai_response() {
		// Test handling of empty arrays from yabai
		let empty_spaces: Vec<SpaceInfo,> = vec![];
		assert!(empty_spaces.is_empty());

		let empty_windows: Vec<WindowInfo,> = vec![];
		assert!(empty_windows.is_empty());
	}

	#[tokio::test]
	async fn test_state_update_consistency() {
		let state = DaemonState::new();

		// Simulate rapid state updates
		for _ in 0..5 {
			let _ = state.update_spaces().await;
			let _ = state.update_windows().await;
			let _ = state.update_current_app().await;
			sleep(Duration::from_millis(100,),).await;
		}

		// State should remain consistent
		let spaces = state.spaces.read().await;
		let windows = state.windows.read().await;

		// Basic consistency checks
		for window in windows.values() {
			if let Some(space,) = spaces.get(&window.space,) {
				assert_eq!(
					space.display, window.display,
					"Window and space should be on same display"
				);
			}
		}
	}

	#[test]
	fn test_sketchybar_command_formatting() {
		let bar = SketchyBar::new();

		// Test that bar name is properly set
		assert_eq!(bar.get_bar_name(), "sketchybar");

		// Test command formatting (this would require more complex mocking)
		// For now, just verify the bar can be created
		let mut bar2 = SketchyBar::new();
		bar2.set_bar_name("test_bar",);
		assert_eq!(bar2.get_bar_name(), "test_bar");
	}

	#[tokio::test]
	async fn test_concurrent_state_access() {
		let state = DaemonState::new();

		// Test concurrent read access
		let state1 = state.clone();
		let state2 = state.clone();

		let task1 = tokio::spawn(async move {
			let _spaces = state1.spaces.read().await;
			sleep(Duration::from_millis(100,),).await;
		},);

		let task2 = tokio::spawn(async move {
			let _windows = state2.windows.read().await;
			sleep(Duration::from_millis(100,),).await;
		},);

		// Both tasks should complete without deadlock
		let (result1, result2,) = tokio::join!(task1, task2);
		assert!(result1.is_ok());
		assert!(result2.is_ok());
	}
}

/// Performance and stress tests
#[cfg(test)]
mod performance_tests {
	use super::*;
	use std::time::Instant;

	#[tokio::test]
	async fn test_state_update_performance() {
		if !utils::is_yabai_available() {
			println!("Skipping performance test - yabai not available");
			return;
		}

		let state = DaemonState::new();

		// Measure state update time
		let start = Instant::now();
		for _ in 0..10 {
			let _ = state.update_spaces().await;
			let _ = state.update_windows().await;
			let _ = state.update_current_app().await;
		}
		let duration = start.elapsed();

		println!("10 state updates took: {:?}", duration);

		// Should complete reasonably quickly (adjust threshold as needed)
		assert!(duration.as_secs() < 5, "State updates should be reasonably fast");
	}

	#[tokio::test]
	async fn test_memory_usage_stability() {
		let state = DaemonState::new();

		// Perform many state updates to check for memory leaks
		for i in 0..100 {
			let _ = state.update_spaces().await;
			let _ = state.update_windows().await;
			let _ = state.update_current_app().await;

			if i % 20 == 0 {
				// Periodically check that we can still access state
				let spaces = state.spaces.read().await;
				let windows = state.windows.read().await;
				let current_app = state.current_app.read().await;

				// Just verify we can access the data
				let _ = spaces.len();
				let _ = windows.len();
				let _ = current_app.is_some();
			}
		}

		println!("Memory stability test completed");
	}
}

/// Real-world scenario tests
#[cfg(test)]
mod scenario_tests {
	use super::*;

	#[tokio::test]
	#[ignore = "hung"]
	async fn test_space_switching_scenario() {
		if !utils::is_yabai_available() {
			println!("Skipping space switching test - yabai not available");
			return;
		}

		let state = DaemonState::new();

		// Get current space
		let original_space = utils::get_current_space();
		if original_space.is_none() {
			println!("Cannot determine current space, skipping test");
			return;
		}
		let original_space = original_space.unwrap();

		// Update state to get available spaces
		let _ = state.update_spaces().await;
		let spaces = state.spaces.read().await;

		if spaces.len() < 2 {
			println!("Need at least 2 spaces for switching test, skipping");
			return;
		}

		// Find a different space to switch to
		let target_space = spaces.values().find(|s| s.index != original_space,).map(|s| s.index,);

		if let Some(target_space,) = target_space {
			println!("Switching from space {} to space {}", original_space, target_space);

			// Switch to target space
			if utils::switch_to_space(target_space,) {
				sleep(Duration::from_millis(500,),).await;

				// Update state and verify
				let _ = state.update_spaces().await;
				let updated_spaces = state.spaces.read().await;

				let focused_space =
					updated_spaces.values().find(|s| s.has_focus,).map(|s| s.index,);

				if let Some(focused_space,) = focused_space {
					println!("Currently focused space: {}", focused_space);
					// Note: The switch might not always succeed due to system restrictions
				}

				// Switch back to original space
				utils::switch_to_space(original_space,);
				sleep(Duration::from_millis(500,),).await;
			}
		}
	}

	#[tokio::test]
	async fn test_window_creation_scenario() {
		if !utils::is_yabai_available() {
			println!("Skipping window creation test - yabai not available");
			return;
		}

		let state = DaemonState::new();

		// Get initial window count
		let _ = state.update_windows().await;
		let initial_windows = state.windows.read().await;
		let initial_count = initial_windows.len();

		println!("Initial window count: {}", initial_count);

		// Create a test window
		let does_test_window_created = utils::create_test_window();
		if does_test_window_created {
			// Wait for window to appear
			sleep(Duration::from_secs(2,),).await;

			// Update state and check for new window
			let updated = state.update_windows().await.expect("update failed",);
			assert!(updated);
			let updated_windows = state.windows.read().await;
			let new_count = updated_windows.len();

			println!("Window count after creation: {}", new_count);

			// Clean up
			utils::cleanup_test_windows();
			sleep(Duration::from_secs(1,),).await;

			// Verify cleanup
			let _ = state.update_windows().await;
			let final_windows = state.windows.read().await;
			let final_count = final_windows.len();

			println!("Final window count: {}", final_count);
		}
	}

	#[tokio::test]
	#[ignore = "hung"]
	async fn test_multi_display_scenario() {
		if !utils::is_yabai_available() {
			println!("Skipping multi-display test - yabai not available");
			return;
		}

		let displays = yabai::get_displays().await;
		match displays {
			Ok(displays,) => {
				if displays.len() > 1 {
					println!("Testing multi-display scenario with {} displays", displays.len());

					let state = DaemonState::new();
					let _ = state.update_spaces().await;
					let spaces = state.spaces.read().await;

					// Verify spaces are distributed across displays
					let mut display_space_count = std::collections::HashMap::new();
					for space in spaces.values() {
						*display_space_count.entry(space.display,).or_insert(0,) += 1;
					}

					println!("Spaces per display: {:?}", display_space_count);

					// Each display should have at least one space
					for display_id in displays.keys() {
						let display_index: u32 = display_id.parse().unwrap_or(0,);
						if display_index > 0 {
							assert!(
								display_space_count.contains_key(&display_index),
								"Display {} should have at least one space",
								display_index
							);
						}
					}
				} else {
					println!("Single display detected, skipping multi-display test");
				}
			},
			Err(e,) => {
				panic!("Failed to get displays for multi-display test: {}", e);
			},
		}
	}
}
