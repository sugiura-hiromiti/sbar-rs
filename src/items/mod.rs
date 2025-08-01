pub mod battery;
pub mod clock;
pub mod current_app;
pub mod keyboard;
pub mod space;
pub mod window;

#[cfg(test)] mod tests;

use crate::helpers::yabai::DisplayInfo;
use crate::sketchybar::SketchyBar;
use anyhow::Result;
use tracing::info;

/// Setup all items for a bar based on display type
pub async fn setup_all_items(bar: &mut SketchyBar, display_info: &DisplayInfo,) -> Result<(),> {
	info!("📦 Setting up items for display {}", display_info.index);

	// Always add these items
	clock::setup(bar, display_info,).await?;
	keyboard::setup(bar, display_info,).await?;
	space::setup(bar, display_info,).await?;
	current_app::setup(bar, display_info,).await?;
	window::setup(bar, display_info,).await?;

	// Only add battery to builtin display
	if display_info.is_builtin {
		battery::setup(bar, display_info,).await?;
	}

	info!("✅ All items configured for display {}", display_info.index);
	Ok((),)
}
