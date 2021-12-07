use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    SyntaxKind,
    types::{
        Ident,
        TokenWrapper,
        TypedNode,
    },
};
use crate::scope::Scope;

/// find out if `name` is used in `node`
pub fn find_usage(name: &Ident, node: SyntaxNode<NixLanguage>) -> bool {
    if let Some(scope) = Scope::new(&node) {
        for binding in scope.bindings() {
            if binding.name.as_str() == name.as_str() {
                // shadowed by a a new child scope that redefines the
                // variable with the same name
                return false;
            }
        }
    }

    if node.kind() == SyntaxKind::NODE_IDENT {
        Ident::cast(node).expect("Ident::cast").as_str() == name.as_str()
    } else {
        node.children().any(|node| find_usage(name, node))
    }
}
