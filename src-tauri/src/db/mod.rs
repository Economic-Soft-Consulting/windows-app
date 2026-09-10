//! Database access. Every function here takes `&Connection` rather than `State<Database>`,
//! so the SQL can be exercised in tests without a running Tauri app.

pub mod balances;
pub mod collections;
