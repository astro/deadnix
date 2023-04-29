use crate::{binding::Binding, scope::Scope, usage};
use rnix::{
    NixLanguage,
};
use rowan::{api::SyntaxNode, ast::AstNode};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Clone)]
pub struct DeadCode {
    pub scope: Scope,
    pub binding: Binding,
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Unused {}: {}", self.scope, self.binding.name)
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub no_lambda_arg: bool,
    pub no_lambda_pattern_names: bool,
    pub no_underscore: bool,
}

impl Settings {
    pub fn find_dead_code(&self, node: &SyntaxNode<NixLanguage>) -> Vec<DeadCode> {
        let mut dead = HashSet::new();
        let mut results = HashMap::new();
        // loop so that bodies are ignored that were detected as dead in a
        // previous iteration
        let mut prev_results_len = 1;
        while prev_results_len != results.len() {
            prev_results_len = results.len();
            self.scan(node, &mut dead, &mut results);
        }

        let mut results = results.into_values().collect::<Vec<_>>();
        results.sort_unstable_by_key(|result| result.binding.name.syntax().text_range().start());
        results
    }

    /// recursively scan the AST, accumulating results
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

                    // TODO: combine bindinges and bodies so that
                    // bodies of dead bindings can be marked as dead
                    if binding.is_mortal()
                    && ! scope.bodies().any(|body|
                        // exclude this binding's own node
                        body != binding.decl_node
                        // excluding already unused results
                        && ! dead.contains(&body)
                        // find if used anywhere
                        && usage::find(&binding.name, &body)
                    )
                    && ! binding.has_pragma_skip() {
                        dead.insert(binding.decl_node.clone());
                        results.insert(
                            binding.decl_node.clone(),
                            DeadCode {
                                scope: scope.clone(),
                                binding,
                            },
                        );
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
