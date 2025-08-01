# SketchyBar Daemon 🦀

A complete SketchyBar configuration daemon written in Rust. This project provides a standalone, type-safe alternative to shell script-based SketchyBar configurations with real-time updates, centralized state management, and multi-display support.

## Features

- **🔄 Standalone Daemon**: Runs as a background service, no shell scripts needed
- **📺 Multi-display Support**: Automatically detects and configures bars for builtin and external displays
- **🔄 Dynamic Display Detection**: Monitors for display changes and reconfigures bars automatically
- **⚡ Event-driven Updates**: Centralized state management with efficient update propagation
- **🛡️ Type Safety**: Leverages Rust's type system to prevent configuration errors
- **🔧 Modular Architecture**: Clean separation of concerns with dedicated modules
- **📊 Comprehensive Logging**: Structured logging with tracing for debugging
- **🎨 Catppuccin Theme**: Beautiful color scheme matching your existing setup
- **🔋 Smart Battery Monitoring**: Battery indicator with charging status and color coding
- **🏠 Workspace Management**: Space indicators with yabai integration
- **🕐 Live Clock**: Real-time clock with custom formatting
- **⌨️ Keyboard Layout**: Current input source display
- **📱 App/Window Tracking**: Current application and window information
- **🧪 Unit Testing**: Comprehensive test coverage for core functionality

## Enhanced Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    SketchyBarDaemon                         │
├─────────────────────────────────────────────────────────────┤
│  Event Layer: Centralized event-driven update system       │
├─────────────────────────────────────────────────────────────┤
│  State Layer: Centralized state management (spaces, apps)  │
├─────────────────────────────────────────────────────────────┤
│  Query Layer: Cached yabai/system queries with batching    │
├─────────────────────────────────────────────────────────────┤
│  Items Layer: Individual bar items with state awareness    │
├─────────────────────────────────────────────────────────────┤
│  SketchyBar API: Type-safe command abstraction             │
└─────────────────────────────────────────────────────────────┘
```

```
src/
├── main.rs              # Daemon entry point and orchestration
├── lib.rs               # Library interface for testing
├── events.rs            # Event-driven update system
├── state.rs             # Centralized state management
├── sketchybar.rs        # High-level SketchyBar API wrapper
├── config/              # Bar configuration
│   └── mod.rs          # Bar and default property setup
├── items/               # Individual bar items with update functions
│   ├── mod.rs          # Item orchestration
│   ├── clock.rs        # Time display with real-time updates
│   ├── battery.rs      # Battery status with smart monitoring
│   ├── keyboard.rs     # Input source detection
│   ├── space.rs        # Workspace indicators with yabai integration
│   ├── current_app.rs  # Active application tracking
│   └── window.rs       # Window information display
└── helpers/             # Utility modules
    ├── mod.rs          # Helper module exports
    ├── yabai.rs        # Display detection and yabai integration
    ├── colors.rs       # Catppuccin color palette
    ├── icons.rs        # Nerd Font icon constants
    └── properties.rs   # Configuration property builders
```

### Key Architectural Improvements

1. **Centralized State Management**: The `DaemonState` struct maintains synchronized state for spaces, windows, and applications, reducing redundant yabai queries and improving performance.

2. **Event-driven Updates**: The `EventManager` coordinates all update loops with different frequencies based on item requirements, replacing the previous polling-only approach.

3. **State-aware Items**: Items can now update using centralized state (`update_with_state`) or fall back to direct queries (`update`), providing flexibility and efficiency.

4. **Enhanced Error Resilience**: Individual item failures are isolated and don't crash the entire daemon, with comprehensive error logging.

5. **Comprehensive Testing**: Unit tests cover core functionality, data structures, and state management.

## Installation

### Prerequisites

- macOS with SketchyBar installed (`brew install sketchybar`)
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- yabai (optional, for multi-display and workspace support)
- MesloLGL Nerd Font (for icons)

### Quick Setup

1. **Clone and build the project:**
   ```bash
   cd ~/.config
   git clone <your-repo> sbar-rs
   cd sbar-rs
   cargo build --release
   ```

2. **Install the daemon:**
   ```bash
   # Copy binary to a system location
   sudo cp target/release/sketchybar-daemon /usr/local/bin/
   
   # Or create a symlink
   ln -sf ~/.config/sbar-rs/target/release/sketchybar-daemon /usr/local/bin/sketchybar-daemon
   ```

3. **Create SketchyBar configuration:**
   ```bash
   # Create sketchybarrc that launches the daemon
   mkdir -p ~/.config/sketchybar
   cat > ~/.config/sketchybar/sketchybarrc << 'EOF'
   #!/bin/bash
   # Kill any existing daemon
   pkill -f sketchybar-daemon
   
   # Start the Rust daemon
   exec /usr/local/bin/sketchybar-daemon
   EOF
   chmod +x ~/.config/sketchybar/sketchybarrc
   ```

4. **Start SketchyBar:**
   ```bash
   brew services restart sketchybar
   ```

### Advanced Setup with LaunchAgent

For automatic startup and better process management:

1. **Create LaunchAgent plist:**
   ```bash
   mkdir -p ~/Library/LaunchAgents
   cat > ~/Library/LaunchAgents/com.sketchybar.daemon.plist << 'EOF'
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>com.sketchybar.daemon</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/local/bin/sketchybar-daemon</string>
       </array>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
       <key>StandardOutPath</key>
       <string>/tmp/sketchybar-daemon.log</string>
       <key>StandardErrorPath</key>
       <string>/tmp/sketchybar-daemon.error.log</string>
   </dict>
   </plist>
   EOF
   ```

2. **Load the LaunchAgent:**
   ```bash
   launchctl load ~/Library/LaunchAgents/com.sketchybar.daemon.plist
   ```

## Configuration

### Update System

The daemon uses an event-driven update system with different frequencies:

| Component | Update Method | Frequency | Description |
|-----------|---------------|-----------|-------------|
| **State Sync** | Centralized | 2 seconds | Updates spaces, windows, apps |
| **Clock** | Direct | 1 second | Real-time clock display |
| **Battery** | Direct | 30 seconds | Power-efficient monitoring |
| **Keyboard** | Direct | 5 seconds | Input source changes |
| **Spaces** | State-driven | 1 second | Workspace indicators |
| **Current App** | State-driven | 1 second | Active application |
| **Window** | State-driven | 1 second | Window information |

### Display Configuration

The daemon automatically configures different properties based on display type:

| Property | Builtin Display | External Display |
|----------|----------------|------------------|
| Position | Top | Bottom |
| Height | 56px | 26px |
| Font Size | 16px | 14px |
| Padding | 4px | 2px |
| Corner Radius | 10px | 5px |

### Color Scheme

Uses the Catppuccin color palette:

- **Background**: Surface0 (`0xff313244`)
- **Text**: Text (`0xffcdd6f4`)
- **Accent Colors**: Blue, Green, Yellow, Red based on context
- **Battery**: Color-coded based on charge level and status

## Usage

### Running the Daemon

```bash
# Run directly
sketchybar-daemon

# Run with debug logging
RUST_LOG=debug sketchybar-daemon

# Run with trace logging (very verbose)
RUST_LOG=trace sketchybar-daemon
```

### Testing

```bash
# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Test specific module
cargo test state::tests
```

### Monitoring

```bash
# Check if daemon is running
pgrep -f sketchybar-daemon

# View logs (if using LaunchAgent)
tail -f /tmp/sketchybar-daemon.log

# View error logs
tail -f /tmp/sketchybar-daemon.error.log
```

### Stopping

```bash
# Stop the daemon
pkill -f sketchybar-daemon

# Or if using LaunchAgent
launchctl unload ~/Library/LaunchAgents/com.sketchybar.daemon.plist
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check for errors without building
cargo check
```

### Testing

```bash
# Run with debug output
RUST_LOG=debug cargo run

# Test specific functionality
cargo test

# Test with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out html
```

### Adding New Items

1. Create a new file in `src/items/` (e.g., `cpu.rs`)
2. Implement `setup()` and `update()` functions
3. Optionally implement `update_with_state()` for state-driven updates
4. Add the module to `src/items/mod.rs`
5. Add setup call in `setup_all_items()`
6. Add update task in `EventManager`

Example:
```rust
// src/items/cpu.rs
use anyhow::Result;
use crate::sketchybar::SketchyBar;
use crate::helpers::{colors::Colors, yabai::DisplayInfo};
use crate::state::DaemonState;

pub async fn setup(bar: &mut SketchyBar, display_info: &DisplayInfo) -> Result<()> {
    bar.add("item", "cpu", "right").await?;
    bar.set("cpu", &[
        ("icon", "💻"),
        ("label", "CPU"),
        ("label.color", &format!("0x{:08x}", Colors::GREEN)),
    ]).await?;
    Ok(())
}

pub async fn update(bar: &SketchyBar) -> Result<()> {
    let usage = get_cpu_usage().await?;
    let cmd = format!("--set cpu label={}%", usage);
    bar.message(&cmd).await?;
    Ok(())
}

pub async fn update_with_state(bar: &SketchyBar, state: &DaemonState) -> Result<()> {
    // Use state for more efficient updates
    let usage = calculate_cpu_from_state(state).await?;
    let cmd = format!("--set cpu label={}%", usage);
    bar.message(&cmd).await?;
    Ok(())
}
```

### Customizing Colors

Modify `src/helpers/colors.rs`:

```rust
impl Colors {
    pub const SURFACE0: u32 = 0xff313244; // Change background color
    pub const TEXT: u32 = 0xffcdd6f4;     // Change text color
    // ... other colors
}
```

### Customizing Update Intervals

Modify the intervals in `src/events.rs`:

```rust
// Change clock update to 5 seconds instead of 1
let mut interval = interval(Duration::from_secs(5));
```

## Troubleshooting

### Common Issues

1. **Daemon won't start**
   - Check if SketchyBar is installed: `brew list sketchybar`
   - Verify binary permissions: `ls -la /usr/local/bin/sketchybar-daemon`
   - Check logs: `RUST_LOG=debug sketchybar-daemon`

2. **Items not updating**
   - Check if yabai is running (for spaces/app/window items)
   - Verify system permissions for AppleScript
   - Check error logs for specific item failures

3. **Multiple displays not working**
   - Install yabai: `brew install yabai`
   - Check yabai is running: `yabai -m query --displays`
   - Verify display detection: `RUST_LOG=debug sketchybar-daemon`

4. **High CPU usage**
   - Reduce update intervals in the code
   - Check for infinite loops in update functions
   - Monitor with: `top -p $(pgrep sketchybar-daemon)`

5. **State synchronization issues**
   - Check yabai permissions and functionality
   - Verify JSON parsing with: `yabai -m query --spaces`
   - Enable trace logging: `RUST_LOG=trace sketchybar-daemon`

### Debug Mode

Run with full debug logging:

```bash
RUST_LOG=trace sketchybar-daemon 2>&1 | tee debug.log
```

### Performance Monitoring

```bash
# Monitor resource usage
top -p $(pgrep sketchybar-daemon)

# Check memory usage
ps -o pid,vsz,rss,comm -p $(pgrep sketchybar-daemon)

# Monitor state updates
RUST_LOG=debug sketchybar-daemon | grep "state"
```

## Comparison with Previous Architecture

| Feature | Old (Polling) | New (Event-driven + State) |
|---------|---------------|----------------------------|
| **Performance** | Multiple redundant queries | Centralized state with batched queries |
| **Memory Usage** | Higher (duplicate data) | Lower (shared state) |
| **Update Efficiency** | Fixed intervals for all | Optimized intervals per component |
| **Error Resilience** | Item failures affect others | Isolated error handling |
| **Debugging** | Limited visibility | Comprehensive state logging |
| **Extensibility** | Manual coordination | Event-driven architecture |
| **Testing** | Difficult to test | Unit testable components |

## Future Enhancements

### Phase 2: Advanced Event System
- **SketchyBar Event Subscriptions**: Replace polling with native SketchyBar events
- **Yabai Event Integration**: Real-time space/window change notifications
- **Custom Event System**: User-defined events and triggers

### Phase 3: Configuration System
- **TOML/YAML Configuration**: External configuration files
- **Hot Reloading**: Configuration changes without restart
- **Theme System**: Multiple color schemes and layouts
- **Plugin Architecture**: Dynamic loading of custom items

### Phase 4: Advanced Features
- **Performance Metrics**: Built-in performance monitoring and optimization
- **Web Interface**: Browser-based configuration and monitoring
- **Animation System**: Smooth transitions and visual effects
- **Multi-user Support**: Per-user configurations and state

### Phase 5: Ecosystem Integration
- **Homebrew Formula**: Easy installation via brew
- **Documentation Site**: Comprehensive online documentation
- **Community Plugins**: Plugin marketplace and sharing
- **Integration Examples**: Common workflow integrations

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes with tests: `cargo test`
4. Ensure code quality: `cargo clippy && cargo fmt`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Submit a pull request

### Development Guidelines

- **Write Tests**: All new functionality should include unit tests
- **Document Changes**: Update README and inline documentation
- **Follow Rust Conventions**: Use `cargo fmt` and `cargo clippy`
- **Performance Conscious**: Profile changes that affect update loops
- **Error Handling**: Use `Result` types and proper error propagation

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [SketchyBar](https://github.com/FelixKratz/SketchyBar) - The amazing macOS status bar
- [sketchybar-rs](https://github.com/FelixKratz/sketchybar-rs) - Rust bindings for SketchyBar
- [Catppuccin](https://github.com/catppuccin/catppuccin) - The beautiful color scheme
- [yabai](https://github.com/koekeishiya/yabai) - Tiling window manager for macOS
- [Tokio](https://tokio.rs/) - Asynchronous runtime for Rust
