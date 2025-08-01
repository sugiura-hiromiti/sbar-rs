pub mod config;
pub mod events;
pub mod helpers;
pub mod items;
pub mod sketchybar;
pub mod state;

#[cfg(test)] mod tests;

#[cfg(test)]
mod basic_tests {
	use super::*;

	#[tokio::test]
	async fn test_daemon_state_creation() {
		let state = state::DaemonState::new();

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
		let bar = sketchybar::SketchyBar::new();
		assert_eq!(bar.get_bar_name(), "sketchybar");
	}

	#[test]
	fn test_display_info_creation() {
		let display = helpers::yabai::DisplayInfo {
			index:      1,
			is_builtin: true,
			frame:      helpers::yabai::DisplayFrame { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0, },
		};

		assert_eq!(display.index, 1);
		assert!(display.is_builtin);
		assert_eq!(display.frame.w, 1920.0);
	}

	#[test]
	fn test_space_info_creation() {
		let space = state::SpaceInfo {
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
		let window = state::WindowInfo {
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
}
