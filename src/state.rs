use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Centralized state management for the SketchyBar daemon
#[derive(Debug, Clone,)]
pub struct DaemonState {
	pub spaces:      Arc<RwLock<HashMap<u32, SpaceInfo,>,>,>,
	pub windows:     Arc<RwLock<HashMap<u32, WindowInfo,>,>,>,
	pub current_app: Arc<RwLock<Option<String,>,>,>,
	pub displays:    Arc<RwLock<HashMap<String, crate::helpers::yabai::DisplayInfo,>,>,>,
}

impl DaemonState {
	pub fn new() -> Self {
		Self {
			spaces:      Arc::new(RwLock::new(HashMap::new(),),),
			windows:     Arc::new(RwLock::new(HashMap::new(),),),
			current_app: Arc::new(RwLock::new(None,),),
			displays:    Arc::new(RwLock::new(HashMap::new(),),),
		}
	}

	/// Update spaces state from yabai query
	pub async fn update_spaces(&self,) -> Result<bool,> {
		let spaces_data = crate::helpers::yabai::query_spaces().await?;
		let mut spaces = self.spaces.write().await;

		let mut changed = false;

		// Check for changes
		for space in &spaces_data {
			if let Some(existing,) = spaces.get(&space.index,) {
				if existing.has_focus != space.has_focus
					|| existing.windows.len() != space.windows.len()
				{
					changed = true;
				}
			} else {
				changed = true;
			}
		}

		// Update state
		spaces.clear();
		for space in spaces_data {
			spaces.insert(space.index, space,);
		}

		if changed {
			debug!("🏠 Spaces state updated ({} spaces)", spaces.len());
		}

		Ok(changed,)
	}

	/// Update windows state from yabai query
	pub async fn update_windows(&self,) -> Result<bool,> {
		let windows_data = crate::helpers::yabai::query_windows().await?;
		let mut windows = self.windows.write().await;

		let mut changed = false;
		let old_count = windows.len();

		// Update state
		windows.clear();
		for window in windows_data {
			windows.insert(window.id, window,);
		}

		if windows.len() != old_count {
			changed = true;
			debug!("🪟 Windows state updated ({} windows)", windows.len());
		}

		Ok(changed,)
	}

	/// Update current app state
	pub async fn update_current_app(&self,) -> Result<bool,> {
		let new_app = crate::helpers::yabai::query_focused_app()
			.await
			.unwrap_or_else(|_| "Unknown".to_string(),);

		let mut current_app = self.current_app.write().await;
		let changed = current_app.as_ref() != Some(&new_app,);

		if changed {
			*current_app = Some(new_app.clone(),);
			debug!("📱 Current app updated: {}", new_app);
		}

		Ok(changed,)
	}

	/// Get current focused space
	pub async fn get_focused_space(&self,) -> Option<SpaceInfo,> {
		let spaces = self.spaces.read().await;
		spaces.values().find(|s| s.has_focus,).cloned()
	}

	/// Get spaces for a specific display
	pub async fn get_spaces_for_display(&self, display_index: u32,) -> Vec<SpaceInfo,> {
		let spaces = self.spaces.read().await;
		spaces.values().filter(|s| s.display == display_index,).cloned().collect()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct SpaceInfo {
	pub index:     u32,
	pub display:   u32,
	pub has_focus: bool,
	pub windows:   Vec<u32,>,
	pub label:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct WindowInfo {
	pub id:        u32,
	pub app:       String,
	pub title:     String,
	pub space:     u32,
	pub display:   u32,
	pub has_focus: bool,
}
