use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::time::Duration;
use tokio::time::interval;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::sketchybar::SketchyBar;
use crate::state::DaemonState;

/// Event-driven update system for SketchyBar items
pub struct EventManager {
	state:       DaemonState,
	bars:        Arc<RwLock<HashMap<String, SketchyBar,>,>,>,
	shutdown_rx: broadcast::Receiver<(),>,
}

impl EventManager {
	pub fn new(
		state: DaemonState,
		bars: Arc<RwLock<HashMap<String, SketchyBar,>,>,>,
		shutdown_rx: broadcast::Receiver<(),>,
	) -> Self {
		Self { state, bars, shutdown_rx, }
	}

	/// Start all event-driven update loops
	pub async fn start_event_loops(&mut self,) -> Result<(),> {
		info!("🎯 Starting event-driven update system");

		// State synchronization task - updates centralized state
		let state_sync_task = self.spawn_state_sync_task();

		// Item update tasks - react to state changes
		let clock_task = self.spawn_clock_task();
		let battery_task = self.spawn_battery_task();
		let keyboard_task = self.spawn_keyboard_task();
		let space_task = self.spawn_space_task();
		let app_task = self.spawn_app_task();
		let window_task = self.spawn_window_task();

		// Wait for shutdown or task completion
		tokio::select! {
			_ = self.shutdown_rx.recv() => {
				info!("🛑 Event loops shutting down");
			}
			result = state_sync_task => {
				if let Err(e) = result {
					error!("❌ State sync task error: {}", e);
				}
			}
			result = clock_task => {
				if let Err(e) = result {
					error!("❌ Clock task error: {}", e);
				}
			}
			result = battery_task => {
				if let Err(e) = result {
					error!("❌ Battery task error: {}", e);
				}
			}
			result = keyboard_task => {
				if let Err(e) = result {
					error!("❌ Keyboard task error: {}", e);
				}
			}
			result = space_task => {
				if let Err(e) = result {
					error!("❌ Space task error: {}", e);
				}
			}
			result = app_task => {
				if let Err(e) = result {
					error!("❌ App task error: {}", e);
				}
			}
			result = window_task => {
				if let Err(e) = result {
					error!("❌ Window task error: {}", e);
				}
			}
		}

		Ok((),)
	}

	/// State synchronization task - keeps centralized state up to date
	fn spawn_state_sync_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let state = self.state.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(2,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						// Update all state in parallel
						let (spaces_changed, windows_changed, app_changed) = tokio::join!(
							state.update_spaces(),
							state.update_windows(),
							state.update_current_app()
						);

						if let Err(e) = spaces_changed {
							debug!("Spaces update failed: {}", e);
						}
						if let Err(e) = windows_changed {
							debug!("Windows update failed: {}", e);
						}
						if let Err(e) = app_changed {
							debug!("App update failed: {}", e);
						}
					}
					_ = shutdown_rx.recv() => {
						info!("📊 State sync task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// Clock update task (high frequency)
	fn spawn_clock_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(1,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::clock::update(bar).await {
								error!("❌ Clock update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("🕐 Clock update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// Battery update task (low frequency)
	fn spawn_battery_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(30,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::battery::update(bar).await {
								error!("❌ Battery update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("🔋 Battery update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// Keyboard update task (medium frequency)
	fn spawn_keyboard_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(5,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::keyboard::update(bar).await {
								error!("❌ Keyboard update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("⌨️  Keyboard update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// Space update task (state-driven)
	fn spawn_space_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let state = self.state.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(1,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::space::update_with_state(bar, &state).await {
								error!("❌ Space update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("🏠 Space update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// App update task (state-driven)
	fn spawn_app_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let state = self.state.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(1,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::current_app::update_with_state(bar, &state).await {
								error!("❌ Current app update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("📱 Current app update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}

	/// Window update task (state-driven)
	fn spawn_window_task(&self,) -> tokio::task::JoinHandle<Result<(),>,> {
		let bars = self.bars.clone();
		let state = self.state.clone();
		let mut shutdown_rx = self.shutdown_rx.resubscribe();

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(1,),);

			loop {
				tokio::select! {
					_ = interval.tick() => {
						let bars_guard = bars.read().await;
						for bar in bars_guard.values() {
							if let Err(e) = crate::items::window::update_with_state(bar, &state).await {
								error!("❌ Window update error: {}", e);
							}
						}
					}
					_ = shutdown_rx.recv() => {
						info!("🪟 Window update task shutting down");
						break;
					}
				}
			}
			Ok((),)
		},)
	}
}
