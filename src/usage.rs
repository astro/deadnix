use crate::scope::Scope;
use rnix::{
    types::{Ident, TokenWrapper, TypedNode},
    NixLanguage, SyntaxKind,
};
use rowan::api::SyntaxNode;

/// find out if `name` is used in `node`
pub fn find(name: &Ident, node: &SyntaxNode<NixLanguage>) -> bool {
    if let Some(scope) = Scope::new(node) {
        if scope.inherits_from(name) {
            return true;
        }

        for binding in scope.bindings() {
            if binding.name.as_str() == name.as_str() {
                // shadowed by a a new child scope that redefines the
                // variable with the same name
                return false;
            }
        }
    }

    if node.kind() == SyntaxKind::NODE_IDENT {
        Ident::cast(node.clone()).expect("Ident::cast").as_str() == name.as_str()
    } else {
        node.children().any(|node| find(name, &node))
    }
}
