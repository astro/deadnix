use crate::scope::Scope;
use rnix::{
    ast::Ident,
    NixLanguage, SyntaxKind,
};
use rowan::{api::SyntaxNode, ast::AstNode};

/// find out if `name` is used in `node`
pub fn find(name: &Ident, node: &SyntaxNode<NixLanguage>) -> bool {
    if let Some(scope) = Scope::new(node) {
        if scope.inherits_from(name) {
            return true;
        }

        for binding in scope.bindings() {
            if binding.name.syntax().text() == name.syntax().text() {
                // shadowed by a a new child scope that redefines the
                // variable with the same name
                return false;
            }
        }
    }

    let ident = if node.kind() == SyntaxKind::NODE_IDENT {
        if let Some(ident) = Ident::cast(node.clone()) {
            Some(ident)
        } else {
            None
        }
    } else {
        None
    };
    if let Some(ident) = &ident {
        ident.syntax().text() == name.syntax().text()
    } else {
        node.children().any(|node| find(name, &node))
    }
}
