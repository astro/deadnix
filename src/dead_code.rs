use std::{
    collections::HashMap,
    fmt,
};
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    types::{TokenWrapper, TypedNode},
};
use crate::{
    binding::Binding,
    scope::Scope,
    usage::find_usage,
};


#[derive(Debug, Clone)]
pub struct DeadCode {
    pub scope: Scope,
    pub binding: Binding,
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} {}", self.scope, self.binding.name.as_str())
    }
}

pub fn find_dead_code(node: SyntaxNode<NixLanguage>) -> Vec<DeadCode> {
    let mut results = HashMap::new();
    // loop so that bodies are ignored that were detected as dead in a
    // previous iteration
    let mut prev_results_len = 1;
    while prev_results_len != results.len() {
        prev_results_len = results.len();
        scan(node.clone(), &mut results);
    }

    let mut results = results.into_values()
        .collect::<Vec<_>>();
    results.sort_unstable_by_key(|result|
        result.binding.name.node().text_range().start()
    );
    results
}

/// recursively scan the AST, accumulating results
fn scan(node: SyntaxNode<NixLanguage>, results: &mut HashMap<SyntaxNode<NixLanguage>, DeadCode>) {
    if let Some(scope) = Scope::new(&node) {
        for binding in scope.bindings() {
            if binding.is_mortal()
            && ! scope.bodies().any(|body|
                // exclude this binding's own node
                body != binding.node &&
                // excluding already unused results
                results.get(&body).is_none() &&
                // find if used anywhere
                find_usage(&binding.name, body)
            ) {
                results.insert(binding.node.clone(), DeadCode {
                    scope: scope.clone(),
                    binding: binding,
                });
            }
        }
    }

    for child in node.children() {
        scan(child, results);
    }
}
