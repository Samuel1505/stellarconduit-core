# Implement SQLite storage for mesh graph and message queue

Closes #26

## Implementation Details

- **`Cargo.toml`**
  - Added `rusqlite` (bundled feature) and `tokio-rusqlite` version 0.5.
- **`src/persistence/db.rs`**
  - Created `MeshDatabase` wrapper over an async SQLite connection. 
  - Schema defined for `peers` (with rep, bans, transports, telemetry), `topology_edges`, and `pending_messages` tables.
  - Implemented async mapping: `save_peer`, `load_all_peers`, `save_envelope`, `load_pending_envelopes`, `delete_envelope`.
  - Also implemented `mark_peer_offline`, `delete_messages_older_than`, and `upsert_edge` to fulfill expectations of `src/topology/health.rs`. 
- **`src/persistence/errors.rs`**
  - Defined customized `DbError` handling SQLite IO and rmp-serde serialization wraps.
- **Tests**
  - Included `tests/persistence_test.rs` simulating asynchronous peer updates, message queue retention and deletion using a `:memory:` runtime. All suite commands executed perfectly.
