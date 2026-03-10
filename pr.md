# feat(discovery): Implement WiFi-Direct service discovery via mDNS

## Summary

Implements WiFi-Direct / local network peer discovery using mDNS (Multicast DNS / Zeroconf) in `src/discovery/wifi_direct.rs`. When devices form a WiFi-Direct P2P group and land on a private 192.168.x.x subnet, they can now discover each other by registering and browsing for `_stellarconduit._tcp.local.` services using the `mdns-sd` crate.

## Context

Once a WiFi-Direct P2P group is formed (two Android devices negotiate a group), both devices land on a private 192.168.x.x subnet. Plain BLE discovery doesn't know about this network topology. mDNS allows each device to broadcast its presence with its identity payload and TCP port, so the other device can connect via `WifiDirectConnection` (Issue #14) without knowing the IP address in advance.

## Changes

### New Files
- `src/discovery/wifi_direct.rs` - Complete mDNS discovery implementation
- `src/discovery/errors.rs` - `DiscoveryError` type for discovery-related errors

### Modified Files
- `Cargo.toml` - Added `mdns-sd = "0.11"` dependency
- `src/discovery/mod.rs` - Exported `wifi_direct` and `errors` modules

### Implementation Details

#### `MdnsAdvertiser`
- Registers an mDNS service of type `_stellarconduit._tcp.local.`
- Encodes peer identity and capabilities in TXT records:
  - `pubkey=<hex>`: 64-character hex-encoded Ed25519 public key
  - `is_relay=<0|1>`: Relay capability flag
- Automatically uses local IP addresses for service registration
- Provides `start()` and `stop()` methods for lifecycle management

#### `MdnsScanner`
- Browses for `_stellarconduit._tcp.local.` services on the local subnet
- Spawns background async task to process service discovery events
- Parses TXT records to extract peer identity
- Integrates with `PeerList` to generate `DiscoveryEvent::PeerDiscovered` events
- Handles service resolution, updates, and removal events

#### Error Handling
- New `DiscoveryError` enum with variants for:
  - mDNS daemon errors
  - Service registration failures
  - Service browsing failures
  - Invalid TXT record formats
  - Pubkey parsing errors

## Testing

- ✅ 7 new unit tests covering:
  - TXT record parsing (valid, missing, malformed, wrong length)
  - Advertiser service registration
  - Scanner browsing functionality
- ✅ All 108 existing tests still pass
- ✅ Code compiles without errors or warnings

## API Usage

```rust
use stellarconduit_core::discovery::wifi_direct::{MdnsAdvertiser, MdnsScanner};
use stellarconduit_core::peer::identity::PeerIdentity;

// Start advertising
let identity = PeerIdentity::new(pubkey);
let advertiser = MdnsAdvertiser::start(8080, identity, false)?;

// Start scanning
let peer_list = Arc::new(Mutex::new(PeerList::new(300)));
let scanner = MdnsScanner::start(peer_list)?;

// ... later ...
advertiser.stop();
scanner.stop();
```

## Acceptance Criteria

- ✅ `MdnsAdvertiser` registers a service visible on the local network via `dns-sd -B _stellarconduit._tcp`
- ✅ `MdnsScanner` fires a `PeerDiscovered` event when another `MdnsAdvertiser` on the same subnet starts
- ✅ Unit tests using mdns-sd's built-in test utilities verify service registration and resolution

## Dependencies

- `mdns-sd = "0.11"` - mDNS service discovery library (RFC 6762/6763 compliant)

## Related Issues

Closes #14 (WiFi-Direct connection support)

## Notes

- The implementation uses `HashMap<String, String>` for TXT properties as required by mdns-sd 0.11 API
- Service names are generated using the first 16 characters of the peer's hex-encoded pubkey
- Signal strength is set to 0 for mDNS-discovered peers (not available from mDNS)
- The implementation gracefully handles environments where mDNS daemon creation may fail (e.g., CI without network)
