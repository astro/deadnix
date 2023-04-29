//! Scan `.nix` files for dead code (unused variable bindings).

mod binding;
mod dead_code;
mod dead_code_tests;
mod edit;
mod edit_tests;
pub mod report;
mod scope;
mod usage;

pub use binding::Binding;
pub use scope::Scope;
pub use dead_code::{DeadCode, Settings};
pub use edit::edit_dead_code;
