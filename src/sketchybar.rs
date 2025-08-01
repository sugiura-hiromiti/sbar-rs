use anyhow::Result;
use sketchybar_rs::message;
use tracing::debug;

/// High-level wrapper around the sketchybar-rs message function
#[derive(Clone, Debug,)]
pub struct SketchyBar {
	bar_name: String,
}

impl SketchyBar {
	pub fn new() -> Self {
		Self { bar_name: "sketchybar".to_string(), }
	}

	pub fn set_bar_name(&mut self, name: &str,) {
		self.bar_name = name.to_string();
	}

	pub fn get_bar_name(&self,) -> &str {
		&self.bar_name
	}

	/// Send a raw message to sketchybar
	/// FIX: `sketchybar_rs::message` does not return
	pub async fn message(&self, msg: &str,) -> Result<String,> {
		debug!("Sending message to {}: {}", self.bar_name, msg);

		let response = message(msg, Some(&self.bar_name,),)
			.map_err(|e| anyhow::anyhow!("SketchyBar error for '{}': {}", self.bar_name, e),)?;

		Ok(response,)
	}

	/// Send a synchronous message (for use in closures)
	pub fn message_sync(&self, msg: &str,) -> Result<String,> {
		debug!("Sending sync message to {}: {}", self.bar_name, msg);

		let response = message(msg, Some(&self.bar_name,),)
			.map_err(|e| anyhow::anyhow!("SketchyBar sync error for '{}': {}", self.bar_name, e),)?;

		Ok(response,)
	}

	/// Configure bar properties
	pub async fn bar(&mut self, properties: &[(&str, &str,)],) -> Result<(),> {
		let mut cmd = "--bar".to_string();
		for (key, value,) in properties {
			cmd.push_str(&format!(" {}={}", key, value),);
		}
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Set default properties for items
	pub async fn default(&mut self, properties: &[(&str, &str,)],) -> Result<(),> {
		let mut cmd = "--default".to_string();
		for (key, value,) in properties {
			cmd.push_str(&format!(" {}={}", key, value),);
		}
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Add an item to the bar
	pub async fn add(&mut self, item_type: &str, name: &str, position: &str,) -> Result<(),> {
		let cmd = format!("--add {} {} {}", item_type, name, position);
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Set properties for an item
	pub async fn set(&mut self, item_name: &str, properties: &[(&str, &str,)],) -> Result<(),> {
		let mut cmd = format!("--set {}", item_name);
		for (key, value,) in properties {
			cmd.push_str(&format!(" {}={}", key, value),);
		}
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Subscribe an item to events
	pub async fn subscribe(&mut self, item_name: &str, events: &[&str],) -> Result<(),> {
		let mut cmd = format!("--subscribe {}", item_name);
		for event in events {
			cmd.push_str(&format!(" {}", event),);
		}
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Enable or disable hotloading
	pub async fn hotload(&mut self, enabled: bool,) -> Result<(),> {
		let cmd = format!("--hotload {}", enabled);
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Remove an item from the bar
	pub async fn remove(&mut self, item_name: &str,) -> Result<(),> {
		let cmd = format!("--remove {}", item_name);
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Query information from sketchybar
	pub async fn query(&self, query_type: &str, item_name: Option<&str,>,) -> Result<String,> {
		let cmd = if let Some(name,) = item_name {
			format!("--query {} {}", query_type, name)
		} else {
			format!("--query {}", query_type)
		};
		self.message(&cmd,).await
	}

	/// Trigger an event
	pub async fn trigger(&self, event_name: &str,) -> Result<(),> {
		let cmd = format!("--trigger {}", event_name);
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Update the bar (force refresh)
	pub async fn update(&self,) -> Result<(),> {
		let cmd = "--update".to_string();
		self.message(&cmd,).await?;
		Ok((),)
	}

	/// Reload the entire bar configuration
	pub async fn reload(&self,) -> Result<(),> {
		let cmd = "--reload".to_string();
		self.message(&cmd,).await?;
		Ok((),)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sketchybar_rs_behavior() -> Result<(),> {
		let query = message("--query bar", None,)?;
		assert!(query.is_empty());
		Ok((),)
	}
}
