# PDOS — Peer-to-Peer Distributed Operating System

PDOS is a peer-to-peer distributed operating system runtime for local-network peer discovery and communication.

## Code Structure

```
pdos/
├── android/                          # Android platform support (placeholder)
├── docs/                             # Documentation (placeholder)
├── protocol/                         # Protocol specifications (placeholder)
├── runtime/                          # Core Rust runtime
│   ├── Cargo.toml                    # Project manifest (tokio, serde, serde_json)
│   └── src/
│       ├── main.rs                   # Entry point — initializes Config + Runtime
│       ├── runtime/
│       │   ├── mod.rs                # Re-exports Config, Runtime
│       │   ├── app.rs                # Runtime struct — start() loop, discovery
│       │   ├── config.rs             # Node config (node_id, device_name, ports)
│       │   └── event_loop.rs         # Event loop stub (disabled)
│       ├── discovery/
│       │   ├── mod.rs
│       │   └── discovery.rs          # Discovery stubs (new, initialize, handle_packet)
│       ├── models/
│       │   ├── mod.rs
│       │   └── node.rs               # Node model (id, name, ip, port, last_seen)
│       ├── protocol/
│       │   ├── mod.rs
│       │   └── message.rs            # DiscoverMessage with serde + validation
│       ├── registry/
│       │   ├── mod.rs
│       │   └── registry.rs           # In-memory HashMap<Node> with CRUD operations
│       ├── transport/
│       │   ├── mod.rs
│       │   └── transport.rs          # UDP multicast send/receive on 224.0.0.167:53317
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
| `runtime` | Core loop — initializes subsystems, orchestrates discovery |
| `discovery` | Peer discovery abstraction (stub) |
| `models` | Data types — `Node` representing a discovered peer |
| `protocol` | Wire format — `DiscoverMessage` with JSON serialization |
| `registry` | In-memory peer store — upsert, query, remove nodes |
| `transport` | UDP multicast networking — bind, receive, broadcast |
| `security` | Security initialisation (stub) |
| `events` | Event types for inner runtime communication |
