# PDOS Android Discovery — Complete Fix Log

**Mac ↔ Samsung S23 Hotspot**  
Cross-platform peer discovery on PDOS v0.1.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────┐
│                  PDOS Runtime                       │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Identity  │  │ Registry │  │  app.rs          │  │
│  │  (crypto) │  │ (nodes)  │  │  (coordinator)   │  │
│  └────┬─────┘  └────┬─────┘  └───────┬──────────┘  │
│       │              │               │              │
│  ┌────▼─────┐  ┌────▼─────┐  ┌──────▼───────────┐  │
│  │ Liveness  │  │ Discovery│  │ Transport        │  │
│  │ (heart)   │  │(strategy)│  │  ┌────────────┐  │  │
│  └────┬─────┘  └────┬─────┘  │  │ send()      │  │  │
│       │              │       │  │ send_to()   │  │  │
│       │              │       │  │ receive()   │  │  │
│       │              │       │  └────────────┘  │  │
│       │     ┌────────▼──┐    │                   │  │
│       │     │  decide   │    │  (no discovery    │  │
│       │     │  strategy │    │   logic — just    │  │
│       │     │  (future) │    │   raw UDP)        │  │
│       │     └───────────┘    └───────────────────┘  │
│  ┌─────────────────────────┐                        │
│  │   ffi/android.rs        │  JNI bridge            │
│  │   Android ↔ Rust        │                        │
│  └─────────────────────────┘                        │
└────────────────────────────────────────────────────┘
```

---

## Problem

PDOS uses **UDP multicast** (`239.255.100.100:55317`) for peer discovery.
This fails on Android for two independent reasons:

| Problem | Root Cause | Symptom |
|---------|-----------|---------|
| **Runtime crash** | `home_dir().expect()` panics (no home dir on Android) | Rust thread silently dies, no runtime |
| **No logs** | `println!` invisible on Android | Can't debug anything |
| **Multicast blocked** | Android WiFi ignores multicast without explicit lock | Packets from Mac never reach Android |
| **AP-side broadcast block** | Samsung hotspot does not bridge broadcast/multicast *from* the phone *to* hotspot clients | Android's response never reaches Mac |
| **Self-packet loop** | `responds_to_discover` flag set before identity filter | Responds to own Discover (wasteful) |
| **Port in use** | `SO_REUSEADDR` not set, `expect()` panics on bind failure | Crash on rapid restart |
| **Global broadcast alone** | Some networks (Samsung hotspot) block `255.255.255.255` | No fallback works |

---

## Fix 1: `APP_DATA_DIR` — Use Android's `filesDir` Instead of `home_dir()`

### Files
- `runtime/src/lib.rs` — global `APP_DATA_DIR` constant
- `runtime/src/ffi/android.rs` — receives `filesDir` from Kotlin, stores in `APP_DATA_DIR`
- `runtime/src/identity/storage.rs` — reads `APP_DATA_DIR` if set, falls back to `home_dir()` otherwise

### What Changed

```rust
// lib.rs — thread-safe global
pub(crate) static APP_DATA_DIR: OnceLock<String> = OnceLock::new();
```

```rust
// ffi/android.rs — JNI entry point receives path
let path: String = env.get_string(&files_dir).expect(...).into();
crate::APP_DATA_DIR.set(path).expect("Data dir already set");
```

```rust
// identity/storage.rs — prefer APP_DATA_DIR over home_dir()
fn identity_file() -> PathBuf {
    let mut path = if let Some(dir) = crate::APP_DATA_DIR.get() {
        PathBuf::from(dir)
    } else {
        home_dir().expect("Unable to locate home directory")
    };
    path.push(".pdos");
    // ...
}
```

### Why This Works
- Android has no `$HOME` — `home_dir()` returns `None`, `expect()` panics.
- The Kotlin `Activity` or `Service` passes `filesDir.absolutePath` via JNI.
- The Rust runtime stores this in a `OnceLock<String>` before spawning the async runtime.
- Identity, registry, and any future file-based storage use this path.

---

## Fix 2: Structured Logging — Replace `println!` with `log`

### Files
- `runtime/Cargo.toml` — add `log`, `android_logger`, `env_logger` dependencies
- `runtime/src/lib.rs` — `init_logging()` with platform-conditional initialization

### Dependencies

```toml
[dependencies]
log = "0.4"

[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.15"

[target.'cfg(not(target_os = "android"))'.dependencies]
env_logger = "0.11"
```

### Initialization

```rust
// lib.rs
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
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("info")
        )
        .format_timestamp(None)
        .init();
    });
}
```

### Usage Pattern

```rust
// Before (invisible on Android):
println!("Runtime starting");
println!("Packet: {}", json);

// After (visible in logcat):
log::info!("Runtime starting...");
log::info!("{}", json);
```

### Why This Works
- `android_logger` routes `log::info!()` etc. to Android's `logcat` under tag `PDOS-RUST`.
- On macOS/Linux, `env_logger` prints to stderr with `RUST_LOG` filter support.
- The platform-conditional `cfg` block means only the relevant logger is compiled.

---

## Fix 3: `WifiManager.MulticastLock` — Enable Multicast Reception on Android

### File
- `android/app/src/main/java/dev/pdos/PDOSApplication.kt`

### Code

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

    override fun onTerminate() {
        multicastLock?.let {
            if (it.isHeld) {
                it.release()
                Log.d("PDOS", "MulticastLock released")
            }
        }
        super.onTerminate()
    }
}
```

### AndroidManifest.xml

```xml
<application
    android:name=".PDOSApplication"
    ...>
```

### Why This Works
- Android WiFi chipset drops all multicast IP traffic by default to save battery.
- `WifiManager.MulticastLock` tells the WiFi stack to keep multicast enabled.
- Acquired in `Application.onCreate()` so it's active before any Activity starts.
- `setReferenceCounted(false)` prevents nested acquire/release issues.

---

## Fix 4: Transport Hardening — Graceful Initialization & Broadcast Fallback

### File
- `runtime/src/transport/transport.rs`

### 4a. `SO_REUSEADDR` on Socket

```rust
let _ = socket.set_reuse_address(true);
```

**Why**: Without `SO_REUSEADDR`, rapid restart of the runtime fails with `AddrInUse` (EADDRINUSE). The previous process's TIME_WAIT state blocks the new bind.

### 4b. Graceful Error Handling (No Panics)

```rust
// Before (panics on any failure):
let socket = Socket::new(...).expect("Failed to create socket");
socket.bind(&addr).expect("Failed to bind");

// After (logs and returns, runtime continues without transport):
let socket = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
    Ok(s) => s,
    Err(e) => {
        log::error!("Failed to create UDP socket: {}", e);
        return;
    }
};

if let Err(e) = socket.bind(&addr) {
    log::error!("Failed to bind UDP socket: {}", e);
    return;
}
```

**Why**: A crashed runtime is useless. Logging the error and returning allows the rest of the system to function (e.g., an API server could still serve status pages).

### 4c. Three-Way Send Fallback Chain

```rust
pub async fn send(&self, packet: &Packet) {
    // 1. Multicast (primary)
    let _ = socket.send_to(&data, &multicast_addr).await;

    // 2. Global broadcast (fallback for networks blocking multicast)
    let global_broadcast = format!("255.255.255.255:{}", constants::MULTICAST_PORT);
    let _ = socket.send_to(&data, &global_broadcast).await;

    // 3. Subnet-directed broadcast (fallback for Samsung hotspot)
    if let Some(local_ip) = Self::find_local_ip() {
        let subnet_bcast = Self::subnet_broadcast(local_ip);
        let _ = socket.send_to(&data, &subnet_bcast).await;
    }
}
```

**Subnet broadcast calculation:**

```rust
fn subnet_broadcast(local_ip: Ipv4Addr) -> String {
    let ip_bits = u32::from(local_ip);
    let broadcast = ip_bits | 0x000000FF;       // /24 assumption
    format!("{}:{}", Ipv4Addr::from(broadcast), constants::MULTICAST_PORT)
}
```

Each send is `let _ =` — silent failure if the network drops it.

### Why the Three Sends Matter

| Send Type | Address | Samsung Hotspot | Corporate WiFi |
|-----------|---------|-----------------|----------------|
| **Multicast** | `239.255.100.100` | ❌ Blocked | ✅ Works |
| **Global broadcast** | `255.255.255.255` | ❌ Blocked | ✅ Works |
| **Subnet broadcast** | `10.93.185.255` (derived) | ✅ Works | ✅ Works |

> **Note:** The subnet broadcast calculation assumes `/24` (`ip \| 0xFF`). This is acceptable for
> v0.1 (hotspots, consumer WiFi are always `/24`), but will need interface netmask discovery
> to handle `/16`, `/20`, `/27`, VPNs, or enterprise networks later.

On the Samsung S23 hotspot, only the subnet-directed broadcast successfully bridged from Mac to Android.

---

## Fix 5: Unicast Discover Response — `send_to()`

### File
- `runtime/src/transport/transport.rs` — `send_to()` method
- `runtime/src/runtime/app.rs` — calls `send_to()` for Discover responses

### New Method on Transport

```rust
pub async fn send_to(&self, packet: &Packet, target: &str) {
    let socket = match self.udp_socket.as_ref() {
        Some(s) => s,
        None => return,
    };

    let data = match serde_json::to_vec(packet) {
        Ok(d) => d,
        Err(_) => return,
    };

    let _ = socket.send_to(&data, target).await;
    log::info!("Unicasted {} bytes to {}", data.len(), target);
}
```

### Usage in app.rs

```rust
if responds_to_discover
    && self.last_discover_response.elapsed() >= DISCOVER_RESPONSE_COOLDOWN
{
    self.last_discover_response = Instant::now();
    let discover = self.create_discover_message();
    let target = format!("{}:{}", sender.ip(), constants::MULTICAST_PORT);
    log::info!("Responding to Discover from {} via unicast", target);
    self.transport.send_to(&Packet::Discover(discover), &target).await;
}
```

### Why Unicast Instead of Broadcast for the Response

```
Hotspot Topology (Samsung S23):

┌────────────────┐
│  S23 (Android) │  ← hotspot AP at 10.93.185.1
│  app process   │
│  socket:0.0.0.0│
└──────┬─────────┘
       │  hotspot bridge (hostapd + iptables)
       │
       │  ┌─→ Mac receives multicast/broadcast from Android?  ──→ NO
       │  │   (AP does NOT bridge phone-originated broadcast)
       │  │
       │  └─→ Mac receives unicast from Android?  ──→ YES
       │      (targeted UDP bypasses the bridge)
       │
       ▼
┌────────────────┐
│  MacBook Air    │  ← hotspot client at 10.93.185.129
│  en0:10.93.x.129│
└────────────────┘
```

On Samsung OneUI hotspot, `iptables` / `ebtables` rules **do not forward broadcast or multicast frames from the phone's own network stack to hotspot clients**. A direct unicast to the client's IP goes through the routing table (which has a direct route for the hotspot subnet) and reaches the client.

---

## Fix 6: Self-Packet Filter — Don't Respond to Own Discover

### File
- `runtime/src/runtime/app.rs`

### Problem

```rust
// Bug: responds_to_discover is set BEFORE dispatch filters self-packets
let responds_to_discover = matches!(packet, Packet::Discover(_));

self.event_loop.dispatch(/* ... */);   // filters by node_id internally

// Dispatch filtered it out (own packet), but we still have the flag
if responds_to_discover {  // ← true even for own packets!
    self.send_packet(/* ... */);  // sends useless self-response
}
```

### Fix

```rust
// Check node_id match in the matches! guard
let responds_to_discover = matches!(
    &packet,
    Packet::Discover(msg) if msg.identity.node_id != self.identity.node_id
);
```

The `if` guard in the `matches!` macro ensures the flag is only set when:
1. The packet is a `Discover`, AND
2. The `node_id` inside it is different from ours (foreign node)

---

## Fix 7: JNI Bridge — Pass `filesDir` from Kotlin to Rust

### Files
- `android/app/src/main/java/dev/pdos/MainActivity.kt` — calls `startRuntime(filesDir)`
- `android/app/src/main/java/dev/pdos/RuntimeService.kt` — same signature
- `android/app/src/main/java/dev/pdos/PDOSNative.kt` — `external fun startRuntime(filesDir: String)`
- `runtime/src/ffi/android.rs` — JNI function receives `JString`

### Kotlin Side

```kotlin
// MainActivity.kt
PDOSNative.startRuntime(filesDir = filesDir.absolutePath)
```

### Rust JNI

```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_startRuntime(
    mut env: JNIEnv,
    _class: JClass,
    files_dir: JString,       // ← received from Kotlin
) {
    let path: String = env
        .get_string(&files_dir)
        .expect("Failed to get files_dir string")
        .into();

    crate::APP_DATA_DIR.set(path).expect("Data dir already set");
    // ...
}
```

---

## Fix 8: Android Hostname — Replace `"localhost"` with Device Model

### Problem

The `hostname` crate returns `"localhost"` on Android (Android has no traditional hostname).

```
"hostname": "localhost"     ← confusing in the device list
```

### Fix

Three files, one global, one JNI call, one collector change:

**`runtime/src/lib.rs`** — Add `DEVICE_NAME` static:
```rust
pub(crate) static DEVICE_NAME: OnceLock<String> = OnceLock::new();
```

**`runtime/src/ffi/android.rs`** — New JNI function `setDeviceName`:
```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_pdos_PDOSNative_setDeviceName(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) {
    let name: String = env.get_string(&name).expect(...).into();
    let _ = crate::DEVICE_NAME.set(name);
}
```

**`android/.../PDOSNative.kt`** — Declare the external function:
```kotlin
external fun setDeviceName(name: String)
```

**`android/.../PDOSApplication.kt`** — Call before runtime starts:
```kotlin
override fun onCreate() {
    super.onCreate()
    PDOSNative.setDeviceName(Build.MODEL)   // e.g. "SM-S911B"
    // ... rest of init
}
```

**`runtime/src/system/collector.rs`** — Use `DEVICE_NAME` when set:
```rust
let hostname = if let Some(name) = crate::DEVICE_NAME.get() {
    name.clone()
} else {
    get().unwrap_or_default().to_string_lossy().to_string()
};
```

### Result

```
"hostname": "SM-S911B"     ← now shows the phone model
```

---

## Fix 9: Registry Exposure — Real `connectedNodes()` via Shared State

### Problem

`connectedNodes()` was hardcoded to return `0`:
```rust
pub extern "system" fn Java_dev_pdos_PDOSNative_connectedNodes(...) -> jint {
    0  // TODO
}
```

### Fix

**`runtime/src/lib.rs`** — Global node list accessible from JNI:
```rust
pub(crate) static NODE_LIST: Mutex<Vec<Node>> = Mutex::new(Vec::new());
```

**`runtime/src/runtime/app.rs`** — Sync registry to global after each event:
```rust
fn sync_node_list(&self) {
    let nodes: Vec<Node> = self.registry.list_nodes().into_iter().cloned().collect();
    if let Ok(mut list) = crate::NODE_LIST.lock() {
        *list = nodes;
    }
}
```

Called from both the receive path and the heartbeat tick:
```rust
// After each network event:
self.registry.update_node_states();
self.sync_node_list();

// On each heartbeat tick:
self.registry.update_node_states();
self.sync_node_list();
```

**`runtime/src/ffi/android.rs`** — Read from the shared state:
```rust
pub extern "system" fn Java_dev_pdos_PDOSNative_connectedNodes(...) -> jint {
    if let Ok(list) = crate::NODE_LIST.lock() {
        list.len() as jint
    } else {
        0
    }
}
```

### Result

- On startup: `Nodes: 0` (correct — no peers yet)
- After discovering a peer: `Nodes: 1`
- The `NODE_LIST` global is updated on every heartbeat tick and every received packet.

> **Next step**: Expose the full node list (name, IP, capabilities) as JSON via a new
> JNI function so the Android UI can render a device list without polling.

---

## Network Flow: Verified Working

```
Time  Android (S23)                         Mac (MacBook Air)
────  ────────────────────────────────────  ──────────────────────────
T+0   App starts                            —
      MulticastLock acquired                 —
      Runtime init complete                  —
      Send Discover → multicast, broadcast   —
      (goes out corporate WiFi, not hotspot) —

T+10  —                                     Runtime init complete
                                             Send Discover → [multicast,
                                               global bcast, subnet bcast]
                                             (goes out en0 over hotspot)

T+10  ← Receive Discover from 10.93.185.129  —
      "New device discovered: MacBook Air"
      Send unicast Discover response →
        to 10.93.185.129:55317

T+10  —                                     ← Receive foreign Discover
                                               from 10.93.185.220 (Android)
                                             "New device discovered: SM-S911B"
                                             Send unicast Discover response →
                                               to 10.93.185.220:55317

T+15  ← Receive Heartbeat from Mac          ← Receive Heartbeat from Android
      Heartbeat every 5s                    Heartbeat every 5s
```

### Logcat Proof (Android)

```
07-20 00:51:10.511  PDOS-RUST: Received 486 bytes from 10.93.185.129:55317
07-20 00:51:10.511  PDOS-RUST: New device discovered: MOHAKs-MacBook-Air.local
07-20 00:51:10.532  PDOS-RUST: Responding to Discover from 10.93.185.129:55317 via unicast
07-20 00:51:10.532  PDOS-RUST: Unicasted 473 bytes to 10.93.185.129:55317
```

### Console Proof (Mac)

```
[INFO] Received 473 bytes from 10.93.185.220:55317
[INFO] New device discovered: SM-S911B
[INFO] Responding to Discover from 10.93.185.220:55317 via unicast
[INFO] Unicasted 486 bytes to 10.93.185.220:55317
```

---

## Files Changed

| File | Change |
|------|--------|
| `runtime/Cargo.toml` | Added `log`, `android_logger`, `env_logger`, `socket2` deps |
| `runtime/src/lib.rs` | Added `APP_DATA_DIR`, `DEVICE_NAME`, `NODE_LIST` globals; `init_logging()` |
| `runtime/src/ffi/android.rs` | JNI bridge accepts `filesDir`, `setDeviceName`; `connectedNodes()` impl |
| `runtime/src/identity/storage.rs` | Uses `APP_DATA_DIR` when available |
| `runtime/src/transport/transport.rs` | `SO_REUSEADDR`, graceful errors, 3-way send, `send_to()` |
| `runtime/src/runtime/app.rs` | Self-packet filter guard, unicast Discover response, `sync_node_list()` |
| `runtime/src/system/collector.rs` | Uses `DEVICE_NAME` for hostname on Android |
| `android/.../PDOSApplication.kt` | `MulticastLock` acquire, `setDeviceName(Build.MODEL)` |
| `android/.../MainActivity.kt` | Pass `filesDir` to JNI |
| `android/.../PDOSNative.kt` | `startRuntime(filesDir: String)`, `setDeviceName(name: String)` |
| `android/.../AndroidManifest.xml` | `android:name=".PDOSApplication"` |

---

## Commit History

```
d803641 fix(hotspot): ensure reliable discovery on Samsung hotspot
1a4e337 feat(android): integrate Android runtime with multicast discovery
5da2454 feat(runtime): complete runtime architecture for v0.1
```

---

## Configuration Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MULTICAST_GROUP` | `239.255.100.100` | IPv4 multicast group |
| `MULTICAST_PORT` | `55317` | UDP port for all PDOS traffic |
| `HEARTBEAT_INTERVAL_SECS` | `5` | Heartbeat every 5 seconds |
| `DISCOVER_RESPONSE_COOLDOWN` | `5` | Rate-limit unicast responses |
| `SUSPECT_TIMEOUT_SECS` | `10` | Node → Suspect after 10s no heartbeat |
| `OFFLINE_TIMEOUT_SECS` | `15` | Node → Offline after 15s no heartbeat |

---

## Assessment

| Dimension | Score |
|-----------|-------|
| Runtime maturity | **9/10** |
| Architecture | **8.5/10** |

Deducted mainly because some discovery decisions (broadcast fallback, strategy selection,
subnet assumptions) are still embedded in the transport layer rather than isolated in a
dedicated discovery strategy component.

---

## Future Improvements (Not Yet Done)

1. **Extract broadcast fallback into Discovery engine**
   Current: `Transport::send()` tries multicast + global broadcast + subnet broadcast.
   Desired: `Discovery` module chooses strategy per network conditions.
   `Transport` should only provide `send()`, `receive()`, and `send_to(address)` —
   it should not decide *where* packets go.

2. **Runtime status publishing**
   Instead of only logging `"Transport unavailable"`, the runtime should publish
   a structured status like `{ Transport: Down, Discovery: Down, Registry: Active }`
   so future UIs can render component health.

3. **Handle interface selection on dual-network phones**
   When phone has both corporate WiFi AND hotspot active, `find_local_ip()`
   may pick the wrong interface. Could use `getsockopt` + routing table
   enumeration to pick the right one.

4. **Full node list via JNI**
   `connectedNodes()` now returns the count. A new function should return the
   complete node list (name, IP, capabilities) as JSON, so the Android UI can
   render a device list without polling.

5. **Move discovery logic out of `app.rs`**
   `app.rs` should become a coordinator, not a container of protocol logic.
   Eventually: `Runtime → Event Loop → Discovery Engine → Transport`.
