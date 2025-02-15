use crate::{binding::Binding, scope::Scope, usage};
use rnix::{ast::Inherit, NixLanguage, SyntaxKind};
use rowan::{api::SyntaxNode, ast::AstNode};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

/// Instance of a dead binding
///
/// Generate them with [`Settings::find_dead_code()`].
#[derive(Debug, Clone)]
pub struct DeadCode {
    /// The [`Scope`] that introduced the [`binding`](`DeadCode::binding`)
    pub scope: Scope,
    /// The [`Binding`] that is found to be unused
    pub binding: Binding,
    /// Used or unused?
    unused: bool,
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.unused {
            write!(fmt, "Unused {}: {}", self.scope, self.binding.name)
        } else {
            write!(fmt, "Used {}: {}", self.scope, self.binding.name)
        }
    }
}

/// Analysis settings
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    /// Ignore `...: ...`
    pub no_lambda_arg: bool,
    /// Ignore `{ ... }: ...`
    pub no_lambda_pattern_names: bool,
    /// Ignore all `Binding` that start with `_`
    pub no_underscore: bool,
    /// Warn on used binding that starts with `_`
    pub warn_used_underscore: bool,
}

impl Settings {
    /// Find unused bindings
    ///
    /// Loops until no more new [`Binding`] is found that is used only
    /// by [`DeadCode`] that was found in a previous iteration.
    pub fn find_dead_code(&self, node: &SyntaxNode<NixLanguage>) -> Vec<DeadCode> {
        let mut dead = HashSet::new();
        let mut results = HashMap::new();
        let mut prev_results_len = 1;
        while prev_results_len != results.len() {
            prev_results_len = results.len();
            self.scan(node, &mut dead, &mut results);
        }

        let mut results = results.into_values().collect::<Vec<_>>();
        results.sort_unstable_by_key(|result| result.binding.name.syntax().text_range().start());
        results
    }

    /// Recursively scan the AST, accumulating results
    fn scan(
        &self,
        node: &SyntaxNode<NixLanguage>,
        dead: &mut HashSet<SyntaxNode<NixLanguage>>,
        results: &mut HashMap<SyntaxNode<NixLanguage>, DeadCode>,
    ) {
        // check the scope of this `node`
        if let Some(scope) = Scope::new(node) {
            if !(self.no_lambda_arg && scope.is_lambda_arg()) {
                for binding in scope.bindings() {
                    if self.no_underscore && binding.starts_with_underscore() {
                        continue;
                    }
                    if self.no_lambda_pattern_names && scope.is_lambda_pattern_name(&binding.name) {
                        continue;
                    }

                    if binding.is_mortal() && !binding.has_pragma_skip() {
                        let unused = scope.bodies().all(|body|
                            // remove this binding's own node
                            body == binding.decl_node
                            // excluding already unused results
                            || dead.contains(&body)
                            || is_dead_inherit(dead, &body)
                            // or not used anywhere
                            || ! usage::find(&binding.name, &body));
                        if unused
                            || (self.warn_used_underscore
                                && binding.name.syntax().text().char_at(0.into()) == Some('_')
                                && !unused)
                        {
                            dead.insert(binding.decl_node.clone());
                            results.insert(
                                binding.decl_node.clone(),
                                DeadCode {
                                    scope: scope.clone(),
                                    binding,
                                    unused,
                                },
                            );
                        }
                    }
                }
            }
        }

        // recurse through the AST
        for child in node.children() {
            self.scan(&child, dead, results);
        }
    }
}

/// is node body (`InheritFrom`) of an inherit clause that contains only dead bindings?
fn is_dead_inherit(
    dead: &HashSet<SyntaxNode<NixLanguage>>,
    node: &SyntaxNode<NixLanguage>,
) -> bool {
    if node.kind() != SyntaxKind::NODE_INHERIT_FROM {
        return false;
    }

    if let Some(inherit) = node.parent().and_then(Inherit::cast) {
        inherit.attrs().all(|attr| dead.contains(attr.syntax()))
    } else {
        false
    }
}
