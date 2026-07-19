# PDOS — Peer-to-Peer Distributed Operating System

PDOS is a peer-to-peer distributed operating system runtime for local-network peer discovery and communication. Built with Rust and Tokio, it uses UDP multicast for automatic discovery of nearby devices.

## Code Structure

```
pdos/
├── android/                          # Android platform support (placeholder)
├── docs/                             # Documentation (placeholder)
├── protocol/                         # Protocol specifications (placeholder)
├── runtime/                          # Core Rust runtime
│   ├── Cargo.toml                    # Project manifest (tokio, serde, uuid, etc.)
│   └── src/
│       ├── main.rs                   # Entry point — declares modules, starts Runtime
│       ├── constants.rs              # Global constants (protocol version, ports, timing)
│       ├── system.rs                 # Platform detection (OS, device type, hostname)
│       ├── runtime/
│       │   ├── mod.rs                # Re-exports Config, Runtime (event_loop disabled)
│       │   ├── app.rs                # Runtime struct — start() loop, heartbeat, discovery
│       │   ├── config.rs             # Node config (node_id, device_name, capabilities)
│       │   └── event_loop.rs         # Event loop stub (disabled, commented out)
│       ├── discovery/
│       │   ├── mod.rs
│       │   └── discovery.rs          # Discovery stubs (new, initialize, handle_packet)
│       ├── models/
│       │   ├── mod.rs
│       │   ├── node.rs               # Node model (id, name, ip, port, last_seen)
│       │   ├── capability.rs         # Capability enum (FileTransfer, Clipboard, etc.)
│       │   ├── device_type.rs        # DeviceType enum (Laptop, Desktop, Phone, etc.)
│       │   └── operating_system.rs   # OperatingSystem enum (MacOS, Windows, Linux, etc.)
│       ├── protocol/
│       │   ├── mod.rs
│       │   └── message.rs            # DiscoverMessage with serde + validation
│       ├── registry/
│       │   ├── mod.rs
│       │   └── registry.rs           # In-memory HashMap<Node> with CRUD + queries
│       ├── transport/
│       │   ├── mod.rs
│       │   └── transport.rs          # UDP multicast send/receive on 239.255.100.100:55317
│       ├── security/
│       │   └── mod.rs                # Security init stub
│       └── events/
│           ├── mod.rs
│           └── runtime_event.rs      # RuntimeEvent enum (NetworkPacket)
└── README.md
```

### Module Responsibilities

| Module | Responsibility |
|---|---|
| `constants` | Global constants — protocol version, heartbeat interval, node timeout, multicast addr/port |
| `system` | Platform detection — resolves device name, OS, and device type at startup |
| `runtime` | Core loop — initializes subsystems, heartbeat, packet processing, stale-node cleanup |
| `discovery` | Peer discovery abstraction (stub) |
| `models` | Data types — `Node`, `Capability`, `DeviceType`, `OperatingSystem` |
| `protocol` | Wire format — `DiscoverMessage` with JSON serialization and validation |
| `registry` | In-memory peer store — upsert, query by capability/type/OS, remove stale nodes |
| `transport` | UDP multicast networking — bind to `239.255.100.100:55317`, receive, broadcast |
| `security` | Security initialisation (stub) |
| `events` | Event types for inner runtime communication |
