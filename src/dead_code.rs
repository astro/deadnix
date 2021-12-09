use crate::{binding::Binding, scope::Scope, usage};
use rnix::{
    types::{TokenWrapper, TypedNode},
    NixLanguage,
};
use rowan::api::SyntaxNode;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone)]
pub struct DeadCode {
    pub scope: Scope,
    pub binding: Binding,
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Unused {}: {}", self.scope, self.binding.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub no_lambda_arg: bool,
    pub no_underscore: bool,
}

impl Settings {
    pub fn find_dead_code(&self, node: &SyntaxNode<NixLanguage>) -> Vec<DeadCode> {
        let mut results = HashMap::new();
        // loop so that bodies are ignored that were detected as dead in a
        // previous iteration
        let mut prev_results_len = 1;
        while prev_results_len != results.len() {
            prev_results_len = results.len();
            self.scan(node, &mut results);
        }

        let mut results = results.into_values().collect::<Vec<_>>();
        results.sort_unstable_by_key(|result| result.binding.name.node().text_range().start());
        results
    }

    /// recursively scan the AST, accumulating results
    fn scan(
        &self,
        node: &SyntaxNode<NixLanguage>,
        results: &mut HashMap<SyntaxNode<NixLanguage>, DeadCode>,
    ) {
        // check the scope of this `node`
        if let Some(scope) = Scope::new(node) {
            if !(self.no_lambda_arg && scope.is_lambda_arg()) {
                for binding in scope.bindings() {
                    if binding.is_mortal()
                        && !(self.no_underscore && binding.name.as_str().starts_with('_'))
                        && !scope.bodies().any(|body|
                        // exclude this binding's own node
                        body != binding.node &&
                        // excluding already unused results
                        results.get(&body).is_none() &&
                        // find if used anywhere
                        usage::find(&binding.name, &body))
                    {
                        results.insert(
                            binding.node.clone(),
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
            self.scan(&child, results);
        }
    }
}
