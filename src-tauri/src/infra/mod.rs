//! Shared plumbing: filesystem layout, PDF conversion, printing, the WME client.
//! Infrastructure concerns that several domains need, owned by none of them.

pub mod paths;
pub mod pdf;
pub mod printer;
