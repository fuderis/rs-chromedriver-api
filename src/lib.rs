#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod error;   pub use error::{ Error, Result };
pub mod prelude;

pub mod session;  pub use session::Session;
