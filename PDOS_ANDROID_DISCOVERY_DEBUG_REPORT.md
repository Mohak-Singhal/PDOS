# PDOS Android Discovery — Complete Forensic Report

## Overview

This report documents the full debugging journey of getting PDOS's peer discovery working on Android. The Mac (macOS) runtime discovered itself fine via local loopback, and `cargo run` on the Mac worked perfectly. But the Android device running via `cargo ndk` + APK could never see or be seen by the Mac — despite being on the same WiFi network.

---

## Table of Contents

1. [Bug #1: Silent Thread Crash — No Home Directory on Android](#bug-1-silent-thread-crash--no-home-directory-on-android)
2. [Bug #2: `println!` is Invisible on Android](#bug-2-println-is-invisible-on-android)
3. [Bug #3: Android Blocks Outgoing Multicast — Broadcast Fallback](#bug-3-android-blocks-outgoing-multicast--broadcast-fallback)
4. [Bug #4: Missing Bidirectional Discover Exchange](#bug-4-missing-bidirectional-discover-exchange)
5. [How Each Bug Was Discovered](#how-each-bug-was-discovered)
6. [Verification: Final Test Output](#verification-final-test-output)
7. [Complete File Change Summary](#complete-file-change-summary)

---

## Bug #1: Silent Thread Crash — No Home Directory on Android

### What happened

The Rust runtime starts in a `std::thread::spawn` inside the JNI function. On non-Android systems, `dirs::home_dir()` returns `Some(path)`. On Android there is no traditional home directory — the function returns `None`. The code then called `.expect("Unable to locate home directory")`, which panicked. Since this is a detached thread, the panic killed the thread silently — the app showed no crash, just never reached the event loop.

### Original code (`runtime/src/identity/storage.rs:8-13`)

```rust
fn identity_file() -> PathBuf {
    let mut path = home_dir().expect("Unable to locate home directory");  // PANICS on Android
    path.push(".pdos");
    fs::create_dir_all(&path).expect("Unable to create PDOS directory");
    path.push("identity.json");
    path
}
```

### How it was found

1. I inserted `panic!("SEND REACHED")` in `transport.rs:send()` — the app didn't crash, proving `send()` was never reached.
2. Code review of `start_runtime()` → `Identity::load()` → `identity_file()` revealed the `home_dir().expect()`.
3. Confirmed by running `adb shell ls /data/data/dev.pdos/files/.pdos/identity.json` — the file didn't exist.

### Fix

**`runtime/src/lib.rs`** — Added a crate-global `OnceLock<String>` to hold an explicit app data directory path:

```rust
use std::sync::{Once, OnceLock};

pub(crate) static APP_DATA_DIR: OnceLock<String> = OnceLock::new();
```

**`runtime/src/ffi/android.rs:9-27`** — The JNI entry point now accepts a `filesDir: String` parameter from Kotlin and stores it in the static before spawning the thread:

```rust
pub extern "system" fn Java_dev_pdos_PDOSNative_startRuntime(
    mut env: JNIEnv,
    _class: JClass,
    files_dir: JString,             // <-- NEW parameter
) {
    crate::init_logging();
    if RUNTIME_STARTED.swap(true, Ordering::SeqCst) {
        log::info!("PDOS Runtime already running");
        return;
    }
    let path: String = env
        .get_string(&files_dir)
        .expect("Failed to get files_dir string")
        .into();
    crate::APP_DATA_DIR.set(path).expect("Data dir already set");
    log::info!("Starting PDOS Runtime...");
    std::thread::spawn(|| { /* ... tokio runtime ... */ });
}
```

**`runtime/src/identity/storage.rs:8-13`** — Falls back to the app data dir when `home_dir()` would return `None`:

```rust
fn identity_file() -> PathBuf {
    let mut path = if let Some(dir) = crate::APP_DATA_DIR.get() {
        PathBuf::from(dir)                          // Android path
    } else {
        home_dir().expect("Unable to locate home directory")  // Desktop fallback
    };
    path.push(".pdos");
    fs::create_dir_all(&path).expect("Unable to create PDOS directory");
    path.push("identity.json");
    path
}
```

**Kotlin side** — The JNI signature changed, so all call sites must pass the files directory:

- `PDOSNative.kt:9`: `external fun startRuntime(filesDir: String)`
- `MainActivity.kt:15`: `PDOSNative.startRuntime(filesDir = filesDir.absolutePath)`
- `RuntimeService.kt:30`: `PDOSNative.startRuntime(filesDir = filesDir.absolutePath)`

---

## Bug #2: `println!` is Invisible on Android

### What happened

On Android, `println!()` in Rust native code writes to stdout, which is typically discarded or sent to `/dev/null` for native libraries loaded into an app process. None of the runtime's debug output appeared in `adb logcat`, making it impossible to tell how far execution reached.

### Original code (throughout the codebase)

```rust
println!("UDP socket listening on {}", constants::MULTICAST_PORT);
println!("Multicasted {} bytes", data.len());
println!("Runtime starting...");
// etc.
```

### How it was found

After fixing Bug #1, I ran `adb logcat -s PDOS` and `adb logcat -s PDOS-RUST` and saw no Rust output, even though `Status: 1` confirmed `startRuntime()` was called. The thread was running but `println!` output was invisible.

### Fix

**`runtime/Cargo.toml`** — Added the `log` crate and platform-specific loggers:

```toml
log = "0.4"
socket2 = "0.5"

[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.15"

[target.'cfg(not(target_os = "android"))'.dependencies]
env_logger = "0.11"
```

**`runtime/src/lib.rs:23-37`** — A once-called `init_logging()` function initializes the correct logger per platform:

```rust
static LOG_INIT: Once = Once::new();

pub fn init_logging() {
    LOG_INIT.call_once(|| {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("PDOS-RUST"),
        );

        #[cfg(not(target_os = "android"))]
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp(None)
            .init();
    });
}
```

All `println!()` calls throughout the codebase were replaced with `log::info!()`:

| File | Lines changed |
|------|-----------|
| `runtime/src/lib.rs` | 4 `println!` → `log::info!` |
| `runtime/src/runtime/app.rs` | 5 `println!` → `log::info!` |
| `runtime/src/transport/transport.rs` | 6 `println!` → `log::info!` |
| `runtime/src/discovery/discovery.rs` | 4 `println!` → `log::info!` |
| `runtime/src/registry/registry.rs` | 5 `println!` → `log::info!` |
| `runtime/src/security/mod.rs` | 1 `println!` → `log::info!` |
| `runtime/src/ffi/android.rs` | 3 `println!` → `log::info!` |
| `runtime/src/identity/storage.rs` | 1 `println!` → `log::info!` |

After this change, all Rust logs appeared in `adb logcat -s PDOS-RUST` with format:
```
07-20 00:14:36.450 I PDOS-RUST: runtime::transport::transport: Multicasted 473 bytes
```

---

## Bug #3: Android Blocks Outgoing Multicast — Broadcast Fallback

### What happened

Even after the runtime started successfully and the event loop reached `transport.send()`, the Android device's multicast packets never reached the Mac. Unicast (tested via `adb shell nc -u 192.168.1.3 55318`) worked perfectly, but multicast to `239.255.100.100` was silently dropped by the network. This was confirmed by:

1. The Mac's Rust runtime received its own loopback but never packets from `192.168.1.10` (Android).
2. A Python listener on the Mac joined `239.255.100.100:55317`—it received the Mac's own packets, but never the Android's.
3. `/proc/net/igmp` on the Android confirmed the device had joined group `239.255.100.100` on `wlan0`.
4. The Android receives the Mac's multicast (proving inbound multicast works), but outbound multicast is dropped.

This is a router/AP configuration issue (likely IGMP snooping or WiFi multicast filtering). Since the network blocks outgoing multicast from wireless clients, Rust-level socket options can't fix it.

### Original code (`runtime/src/transport/transport.rs:14-38`)

```rust
pub async fn initialize(&mut self) {
    use std::net::{Ipv4Addr, UdpSocket as StdUdpSocket};
    let std_socket = StdUdpSocket::bind(format!("0.0.0.0:{}", constants::MULTICAST_PORT))
        .expect("Failed to bind UDP socket");
    std_socket
        .join_multicast_v4(&constants::MULTICAST_GROUP.parse::<Ipv4Addr>().unwrap(), &Ipv4Addr::UNSPECIFIED)
        .expect("Failed to join multicast group");
    std_socket.set_nonblocking(true).expect("Failed to set nonblocking");
    let socket = UdpSocket::from_std(std_socket).expect("Failed to create Tokio socket");
    self.udp_socket = Some(socket);
    log::info!("UDP socket listening on {}", constants::MULTICAST_PORT);
}
```

### How it was found

1. Tested unicast (Python listener on Mac + `adb shell nc -u 192.168.1.3 55318`) — **worked**.
2. Tested multicast (Python listener joined `239.255.100.100` + `adb shell nc -u 239.255.100.100 55317`) — **failed**.
3. Router/AP asymmetry: Mac→Android multicast works, Android→Mac doesn't.

### Fix

**`runtime/src/transport/transport.rs`** — Complete rewrite of transport initialization using `socket2` crate for proper multicast options, plus a **broadcast fallback** on every `send_to()`:

```rust
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub async fn initialize(&mut self) {
    let multicast_addr: Ipv4Addr = constants::MULTICAST_GROUP.parse().expect("...");
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .expect("Failed to create socket");

    socket.bind(&SockAddr::from(std::net::SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED, constants::MULTICAST_PORT,
    ))).expect("Failed to bind UDP socket");

    socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)
        .expect("Failed to join multicast group");

    socket.set_broadcast(true).expect("Failed to set broadcast");         // <-- NEW
    socket.set_multicast_ttl_v4(2).expect("Failed to set multicast TTL"); // <-- NEW
    socket.set_multicast_loop_v4(true).expect("Failed to set loopback");

    // Explicit outgoing multicast interface (NEW)
    if let Some(local_ip) = Self::find_local_ip() {
        socket.set_multicast_if_v4(&local_ip).unwrap_or_else(|_| {
            log::info!("Could not set multicast IF to {}, using default", local_ip);
        });
        log::info!("Set multicast interface to {}", local_ip);
    }

    socket.set_nonblocking(true).expect("Failed to set nonblocking");
    let std_socket: StdUdpSocket = socket.into();
    let tokio_socket = UdpSocket::from_std(std_socket).expect("Failed to create Tokio socket");
    self.udp_socket = Some(tokio_socket);
    log::info!("UDP socket listening on {}", constants::MULTICAST_PORT);
}
```

The `send()` method now sends to **both** the multicast group and the broadcast address:

```rust
pub async fn send(&self, packet: &Packet) {
    let socket = self.udp_socket.as_ref().expect("UDP socket not initialized");
    let data = serde_json::to_vec(packet).expect("Failed to serialize packet");

    let multicast_addr = format!("{}:{}", constants::MULTICAST_GROUP, constants::MULTICAST_PORT);
    socket.send_to(&data, &multicast_addr).await.expect("Failed to send multicast packet");

    // NEW: broadcast fallback for networks that block multicast from WiFi
    let broadcast_addr = format!("255.255.255.255:{}", constants::MULTICAST_PORT);
    let _ = socket.send_to(&data, &broadcast_addr).await;

    log::info!("Multicasted {} bytes", data.len());
}
```

**`runtime/src/transport/transport.rs:16-29`** — Helper to detect the default local IPv4 address:

```rust
fn find_local_ip() -> Option<Ipv4Addr> {
    let temp = StdUdpSocket::bind("0.0.0.0:0").ok()?;
    temp.connect("1.1.1.1:53").ok()?;    // kernel selects best source IP
    if let Ok(local) = temp.local_addr() {
        if let std::net::IpAddr::V4(ip) = local.ip() {
            if !ip.is_loopback() {
                return Some(ip);
            }
        }
    }
    None
}
```

Additionally, the **Android `MulticastLock`** must be acquired before the socket is created, otherwise the WiFi driver may not forward multicast at all:

**`android/app/src/main/java/dev/pdos/PDOSApplication.kt`**:

```kotlin
class PDOSApplication : Application() {
    private var multicastLock: WifiManager.MulticastLock? = null

    override fun onCreate() {
        super.onCreate()
        try {
            val wifi = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
            multicastLock = wifi.createMulticastLock("PDOS")
            multicastLock?.setReferenceCounted(false)
            multicastLock?.acquire()
            Log.d("PDOS", "MulticastLock acquired")
        } catch (e: Throwable) {
            Log.e("PDOS", "Failed to acquire MulticastLock", e)
        }
    }
}
```

---

## Bug #4: Missing Bidirectional Discover Exchange

### What happened

The `Discovery` module (`runtime/src/discovery/discovery.rs:71-84`) handles `Packet::Heartbeat` by calling `registry.heartbeat(&message.node_id)`, which only updates `last_heartbeat` for **already-known** nodes. It does **not** create new nodes. Only `Packet::Discover` triggers `registry.upsert_node()` (line 68), which creates new node entries.

The Android sends a single `Discover` packet at startup (`app.rs:95`). If the Mac starts after this, it never receives the Android's Discover — it only receives heartbeats, which it silently ignores. Without a Discover exchange, the Mac never learns about the Android.

### How it was found

After fixing broadcast delivery (Bug #3), logcat showed:

```
Mac: Received 95 bytes from 192.168.1.10:55317    (heartbeat received via broadcast)
Mac: {"Heartbeat":{"node_id":"Z8uF8DzPxLyxJW4JIERg7zL8cqxdE2Z3IEJUWNOLJ08=",...}}
```

But no "New device discovered" message appeared. Code review of `discovery.rs:71-84` showed `handle_heartbeat` only calls `registry.heartbeat()`, which does a `get_mut` lookup — no-op for unknown nodes.

### Fix

**`runtime/src/runtime/app.rs:117-143`** — When a `Discover` packet is received, re-announce our own Discover. Rate-limited to 5 seconds to prevent infinite reply loops:

```rust
// In the receive branch of the main event loop:
result = self.transport.receive() => {
    if let Some((packet, sender)) = result {
        let responds_to_discover = matches!(packet, Packet::Discover(_));

        self.event_loop.dispatch(RuntimeEvent::NetworkPacket { packet, sender },
            RuntimeContext { ... });

        self.registry.update_node_states();

        // NEW: Respond to Discover packets so late-starting nodes are seen
        if responds_to_discover
            && self.last_discover_response.elapsed() >= DISCOVER_RESPONSE_COOLDOWN
        {
            self.last_discover_response = Instant::now();
            let discover = self.create_discover_message();
            self.send_packet(Packet::Discover(discover)).await;
        }
    }
}
```

The `Runtime` struct gained a `last_discover_response: Instant` field, initialized to a time in the past so the first discover always triggers a response:

```rust
const DISCOVER_RESPONSE_COOLDOWN: Duration = Duration::from_secs(5);

pub struct Runtime {
    // ... existing fields ...
    last_discover_response: Instant,     // NEW
}

impl Runtime {
    pub fn new(...) -> Self {
        Self {
            // ...
            last_discover_response: Instant::now() - DISCOVER_RESPONSE_COOLDOWN,
        }
    }
}
```

---

## How Each Bug Was Discovered

A step-by-step log of the debugging process:

### Step 1: Is `send()` being called?

Inserted `panic!("SEND REACHED")` in `transport.rs:send()` → App opened but didn't crash → `send()` was never reached.

### Step 2: Trace the startup path

Read `lib.rs` → `start_runtime()` → `Identity::load()` → `identity_file()` → `dirs::home_dir().expect(...)`. Android has no home dir → panics silently in detached thread.

**Fix**: Pass `filesDir` from Kotlin, store in `OnceLock`.

### Step 3: Re-test with fix

Fixed identity loading. Re-inserted `panic!("SEND REACHED")`. App still didn't crash — but now we had no visibility into where it was failing.

### Step 4: Add proper logging

Replaced all `println!` with `log::info!`, added `android_logger`. Now `adb logcat -s PDOS-RUST` showed:

```
PDOS Runtime v0.1
Runtime starting...
UDP socket listening on 55317
Multicasted 473 bytes
```

The runtime WAS starting and sending — but only loopback. No cross-device communication.

### Step 5: Test network path

- `ping 192.168.1.10` from Mac → worked.
- Unicast UDP (`adb shell nc -u 192.168.1.3 55318`) → **worked**.
- Multicast UDP (`adb shell nc -u 239.255.100.100 55317`) → **failed** (Mac listener got nothing).

Conclusion: The network/AP blocks outgoing multicast from WiFi clients. Bug #3.

### Step 6: Implement broadcast fallback

Added `SO_BROADCAST`, `IP_MULTICAST_IF`, `IP_MULTICAST_TTL`, and broadcast duplicate delivery. After this, Mac started receiving Android's heartbeat packets.

### Step 7: Still no node creation on Mac

Mac received Android's heartbeat but showed no "New device discovered". Code review of `discovery.rs` → heartbeats only refresh existing nodes, not create them. The Android's initial Discover was sent before the Mac started. Bug #4.

### Step 8: Add Discover response

When any `Discover` is received, respond with our own Discover. After this, both sides discovered each other.

---

## Verification: Final Test Output

### Android logcat (`adb logcat -s PDOS-RUST`)

```
07-20 00:14:43.879 I PDOS-RUST: runtime::transport::transport: Received 486 bytes from 192.168.1.3:55317
07-20 00:14:43.879 I PDOS-RUST: runtime::registry::registry: New device discovered: MOHAKs-MacBook-Air.local
```

### Mac stdout

```
[INFO  runtime::transport::transport] Received 473 bytes from 192.168.1.10:55317
[INFO  runtime::transport::transport] {"Discover":{"identity":{"node_id":"Z8uF8DzPxLyxJW4JIERg7zL8cqxdE2Z3IEJUWNOLJ08="},"device":{"hostname":"localhost","device_type":"Phone",...}}
[INFO  runtime::registry::registry] New device discovered: localhost
```

Both sides confirmed discovery of the other.

---

## Complete File Change Summary

### Rust files (7 files changed)

| File | What changed |
|------|-------------|
| `runtime/Cargo.toml` | Added `log`, `socket2`, `android_logger`, `env_logger` |
| `runtime/src/lib.rs` | Added `APP_DATA_DIR` static, `LOG_INIT`, `init_logging()` |
| `runtime/src/ffi/android.rs` | `startRuntime` now accepts `filesDir`, calls `init_logging()`, uses `log::info!` |
| `runtime/src/identity/storage.rs` | `identity_file()` falls back to `APP_DATA_DIR` before `home_dir()` |
| `runtime/src/transport/transport.rs` | Switched to `socket2`, added broadcast, multicast IF/TTL, `find_local_ip()` |
| `runtime/src/runtime/app.rs` | Added Discover-response-on-receive with rate limiting |
| `runtime/src/discovery/discovery.rs` | `println!` → `log::info!` |
| `runtime/src/registry/registry.rs` | `println!` → `log::info!` |
| `runtime/src/security/mod.rs` | `println!` → `log::info!` |
| `runtime/src/events/event_loop.rs` | Unchanged (already clean) |

### Kotlin files (4 files changed)

| File | What changed |
|------|-------------|
| `android/.../PDOSApplication.kt` | Acquires `WifiManager.MulticastLock` on startup |
| `android/.../PDOSNative.kt` | `startRuntime` now takes `filesDir: String` |
| `android/.../MainActivity.kt` | Passes `filesDir.absolutePath` |
| `android/.../RuntimeService.kt` | Passes `filesDir.absolutePath` |
