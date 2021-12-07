use std::{
    collections::HashSet,
    fmt,
};
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    types::TokenWrapper,
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

impl PartialEq for DeadCode {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
    }
}
impl Eq for DeadCode {}

impl std::hash::Hash for DeadCode {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.binding.name.as_str().hash(hasher);
        self.binding.node.hash(hasher);
    }
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} {}", self.scope, self.binding.name.as_str())
    }
}

pub fn find_dead_code(node: SyntaxNode<NixLanguage>) -> HashSet<DeadCode> {
    // TODO: loop
    let mut results = HashSet::new();
    scan(node, &mut results);
    results
}

/// recursively scan the AST, accumulating results
fn scan(node: SyntaxNode<NixLanguage>, results: &mut HashSet<DeadCode>) {
    if let Some(scope) = Scope::new(&node) {
        for binding in scope.bindings() {
            let result = DeadCode {
                scope: scope.clone(),
                binding,
            };
            if ! scope.bodies().any(|body|
                // exclude this binding's own node
                body != result.binding.node &&
                // excluding already unused results
                results.get(&result).is_none() &&
                // find if used anywhere
                find_usage(&result.binding.name, body)
            ) {
                results.insert(result);
            }
        }
    }

    for child in node.children() {
        scan(child, results);
    }
}
