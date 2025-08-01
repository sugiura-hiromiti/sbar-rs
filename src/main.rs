mod config;
mod events;
mod helpers;
mod items;
mod sketchybar;
mod state;

use anyhow::Result;
use futures::stream::StreamExt;
use signal_hook::consts::SIGTERM;
use signal_hook_tokio::Signals;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::time::Duration;
use tokio::time::interval;
use tracing::error;
use tracing::info;
use tracing::warn;

use helpers::yabai::DisplayInfo;
use sketchybar::SketchyBar;
use state::DaemonState;

/// Main daemon state
#[derive(Debug,)]
pub struct SketchyBarDaemon {
	displays:    Arc<RwLock<HashMap<String, DisplayInfo,>,>,>,
	bars:        Arc<RwLock<HashMap<String, SketchyBar,>,>,>,
	state:       DaemonState,
	shutdown_tx: broadcast::Sender<(),>,
}

impl SketchyBarDaemon {
	pub fn new() -> Self {
		let (shutdown_tx, _,) = broadcast::channel(1,);

		Self {
			displays: Arc::new(RwLock::new(HashMap::new(),),),
			bars: Arc::new(RwLock::new(HashMap::new(),),),
			state: DaemonState::new(),
			shutdown_tx,
		}
	}

	/// Start the daemon
	pub async fn run(&mut self,) -> Result<(),> {
		info!("🦀 Starting SketchyBar Daemon v0.2.0");

		// Setup signal handling
		let mut signals = Signals::new(&[SIGTERM,],)?;
		let shutdown_tx = self.shutdown_tx.clone();

		tokio::spawn(async move {
			while let Some(signal,) = signals.next().await {
				match signal {
					SIGTERM => {
						info!("📡 Received SIGTERM, shutting down gracefully");
						let _ = shutdown_tx.send((),);
						break;
					},
					_ => {},
				}
			}
		},);

		// Initial display detection and bar setup
		self.detect_and_setup_displays().await?;

		// Start display monitoring
		let daemon_clone = Arc::new(RwLock::new(self.clone(),),);
		let monitor_task = tokio::spawn(Self::monitor_displays(daemon_clone,),);

		// Start update loops for all items using event system
		let mut event_manager = events::EventManager::new(
			self.state.clone(),
			self.bars.clone(),
			self.shutdown_tx.subscribe(),
		);
		let update_task = tokio::spawn(async move { event_manager.start_event_loops().await },);

		// Wait for shutdown signal or task completion
		let mut shutdown_rx = self.shutdown_tx.subscribe();
		tokio::select! {
			_ = shutdown_rx.recv() => {
				info!("🛑 Shutdown signal received");
			}
			result = monitor_task => {
				if let Err(e) = result {
					error!("❌ Display monitor task failed: {}", e);
				}
			}
			result = update_task => {
				if let Err(e) = result {
					error!("❌ Update task failed: {}", e);
				}
			}
		}

		info!("✅ SketchyBar Daemon shutdown complete");
		Ok((),)
	}

	/// Detect displays and setup bars
	async fn detect_and_setup_displays(&mut self,) -> Result<(),> {
		let new_displays = helpers::yabai::get_displays().await?;
		let mut displays = self.displays.write().await;
		let mut bars = self.bars.write().await;

		info!("📺 Detected {} displays", new_displays.len());

		// Remove bars for displays that no longer exist
		let mut to_remove = Vec::new();
		for display_id in displays.keys() {
			if !new_displays.contains_key(display_id,) {
				to_remove.push(display_id.clone(),);
			}
		}

		for display_id in to_remove {
			info!("🗑️  Removing bar for disconnected display {}", display_id);
			displays.remove(&display_id,);
			bars.remove(&display_id,);
		}

		// Add bars for new displays
		for (display_id, display_info,) in new_displays.iter() {
			if !displays.contains_key(display_id,) {
				info!("🚀 Setting up bar for new display {}", display_id);

				let bar_name = if display_info.is_builtin {
					"sketchybar".to_string()
				} else {
					format!("external_{}", display_info.index)
				};

				let mut bar = SketchyBar::new();
				bar.set_bar_name(&bar_name,);

				// Configure the bar
				if let Err(e,) = config::setup_bar(&mut bar, &bar_name, display_info,).await {
					error!("❌ Failed to configure bar {}: {}", bar_name, e);
					continue;
				}

				// Add all items
				if let Err(e,) = items::setup_all_items(&mut bar, display_info,).await {
					error!("❌ Failed to setup items for bar {}: {}", bar_name, e);
					continue;
				}

				// Enable hotloading
				if let Err(e,) = bar.hotload(true,).await {
					warn!("⚠️  Failed to enable hotloading for bar {}: {}", bar_name, e);
				}

				bars.insert(display_id.clone(), bar,);
				info!("✅ Bar '{}' configured for display {}", bar_name, display_id);
			}
		}

		// Update state with new displays
		*displays = new_displays.clone();
		*self.state.displays.write().await = new_displays;

		Ok((),)
	}

	/// Monitor displays for changes
	async fn monitor_displays(daemon: Arc<RwLock<Self,>,>,) -> Result<(),> {
		let mut interval = interval(Duration::from_secs(5,),);

		loop {
			interval.tick().await;

			let mut daemon_guard = daemon.write().await;
			if let Err(e,) = daemon_guard.detect_and_setup_displays().await {
				error!("❌ Display detection failed: {}", e);
			}
		}
	}
}

impl Clone for SketchyBarDaemon {
	fn clone(&self,) -> Self {
		Self {
			displays:    self.displays.clone(),
			bars:        self.bars.clone(),
			state:       self.state.clone(),
			shutdown_tx: self.shutdown_tx.clone(),
		}
	}
}

#[tokio::main]
async fn main() -> Result<(),> {
	// Initialize tracing
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "info".into(),),
		)
		.init();

	// Create and run daemon
	let mut daemon = SketchyBarDaemon::new();
	daemon.run().await
}
