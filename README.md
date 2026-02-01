# rhxd - Hotline Connect Server Suite

A modern Rust implementation of the Hotline Connect protocol suite, consisting of:

- **rhxcore** - Core protocol library for building clients, servers, and bots
- **rhxd** - Hotline server daemon
- **rhxtrackd** - Tracker daemon for server discovery

## Features

### Current (MVP)
- ✅ Full Hotline 1.9.x protocol compatibility
- ✅ Legacy client support
- ✅ Login and authentication (legacy XOR password format)
- ✅ User management and sessions
- ✅ Public chat
- ✅ Private messaging
- ✅ File listing
- ✅ SQLite database storage
- ✅ JSON configuration
- ✅ CLI administration tools
- ✅ Cross-platform (Linux, macOS, Windows)

### Planned
- ⏳ File transfers (download/upload with resume)
- ⏳ Folder transfers
- ⏳ News system
- ⏳ Blake3 password hashing (with migration from legacy)
- ⏳ HOPE protocol extensions (encryption)
- ⏳ >4GB file support (via Nostalgia analysis)
- ⏳ Anti-spam and rate limiting
- ⏳ Bandwidth throttling

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rhxd
cd rhxd

# Build all crates
cargo build --release

# Binaries will be in target/release/
```

### Server Setup

```bash
# Initialize a new server
./target/release/rhxd init

# This creates:
# - rhxd.json (configuration)
# - rhxd.db (SQLite database)
# - Default admin account with random password

# Start the server
./target/release/rhxd serve

# Or with custom config
./target/release/rhxd serve --config /path/to/config.json
```

### Tracker Setup

```bash
# Initialize a new tracker
./target/release/rhxtrackd init

# Start the tracker
./target/release/rhxtrackd serve
```

## Configuration

### Server (rhxd.json)

```json
{
  "server": {
    "name": "My Hotline Server",
    "description": "A modern Rust Hotline server",
    "address": "0.0.0.0",
    "port": 5500,
    "max_connections": 100
  },
  "files": {
    "root_path": "./files",
    "max_download_size": 104857600,
    "enable_uploads": true,
    "enable_downloads": true
  },
  "database": {
    "path": "./rhxd.db"
  },
  "security": {
    "require_login": true,
    "allow_guest": false
  }
}
```

### Tracker (rhxtrackd.json)

```json
{
  "server": {
    "name": "My Hotline Tracker",
    "address": "0.0.0.0",
    "port": 5498
  },
  "http": {
    "enabled": true,
    "address": "0.0.0.0",
    "port": 8080
  },
  "registry": {
    "server_ttl_seconds": 3600,
    "cleanup_interval_seconds": 300
  }
}
```

## CLI Usage

### Server Commands

```bash
# Account management
rhxd account add <login> <password> <name> [--admin]
rhxd account list
rhxd account delete <login>
rhxd account set-password <login> <new-password>

# Database operations
rhxd db migrate
rhxd db index-files <directory>
rhxd db backup <output-file>

# Server info
rhxd info
rhxd version
```

### Tracker Commands

```bash
# Server registry
rhxtrackd server list
rhxtrackd server remove <server-id>

# Database operations
rhxtrackd db cleanup
rhxtrackd db backup <output-file>

# Tracker info
rhxtrackd info
```

## Development

### Building rhxcore Library

```bash
cd crates/rhxcore
cargo build
cargo test
```

### Using rhxcore in Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
rhxcore = { git = "https://github.com/yourusername/rhxd", package = "rhxcore" }
```

Example bot:

```rust
use rhxcore::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to server
    let mut client = HotlineClient::connect("127.0.0.1:5500").await?;
    
    // Login
    client.login("bot", "password").await?;
    
    // Send chat message
    client.send_chat("Hello from Rust!").await?;
    
    Ok(())
}
```

## Architecture

```
rhxd/
├── crates/
│   ├── rhxcore/       # Protocol library (reusable)
│   ├── rhxd/          # Server daemon
│   └── rhxtrackd/     # Tracker daemon
├── docs/              # Documentation
└── examples/          # Example bots and clients
```

### Protocol Support

- **Hotline 1.9.x** - Full compatibility
- **HOPE** (Hotline Open Protocol Extensions) - Planned
- **Legacy clients** - Fully supported (Hotline 1.2.3+)

## Documentation

- [Protocol Documentation](docs/PROTOCOL.md)
- [API Documentation](docs/API.md)
- [Migration Guide](docs/MIGRATION.md) - Migrating from mhxd/GLoarbLine

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Original Hotline protocol by Hotsprings Inc.
- mhxd and hxd developers for their extensive work on the protocol
- GLoarbLine project for the reference implementation
- Nostalgia developer (RIP) for pioneering >4GB file transfers

## Project Status

This project is in active development. The MVP (login, chat, file listing) is being implemented.
File transfers and advanced features will be added in subsequent phases.

## Resources

- [Hotline Wiki](https://hlwiki.com/)
- [Protocol Specification](https://hlwiki.com/index.php/Protocol)
- [GLoarbLine Source](https://github.com/Schala/GLoarbLine)
- [mhxd Source](https://github.com/Schala/mhxd)
