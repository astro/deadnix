//! Scan `.nix` files for dead code (unused variable bindings).
//!
//! ```
//! let content = "
//!     let
//!       foo = {};
//!       inherit (foo) bar baz;
//!     in baz
//! ";
//! let ast = rnix::Root::parse(content);
//! assert_eq!(0, ast.errors().len());
//!
//! let results = deadnix::Settings {
//!     no_lambda_arg: false,
//!     no_lambda_pattern_names: false,
//!     no_underscore: false,
//!     warn_used_underscore: false,
//! }.find_dead_code(&ast.syntax());
//!
//! for dead_code in &results {
//!     println!("unused binding: {}", dead_code.binding.name);
//! }
//! ```

#![deny(unsafe_code, missing_docs, bare_trait_objects)]

mod binding;
mod dead_code;
mod dead_code_tests;
mod edit;
mod edit_tests;
pub mod report;
mod scope;
mod usage;

pub use binding::Binding;
pub use dead_code::{DeadCode, Settings};
pub use edit::edit_dead_code;
pub use scope::Scope;
