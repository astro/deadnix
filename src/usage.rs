use crate::scope::Scope;
use rnix::{ast::Ident, NixLanguage, SyntaxKind};
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

        scope.bodies().any(|node| find(name, &node))
    } else if node.kind() == SyntaxKind::NODE_IDENT {
        // Ident node: occurrence?
        let ident = Ident::cast(node.clone()).unwrap();
        ident.syntax().text() == name.syntax().text()
    } else if node.kind() == SyntaxKind::NODE_ATTRPATH {
        // Don't search for idents in keys, they introduce new scopes
        // anyway. Except for `${...}` and `"..."` which do not
        // introduce new scopes in attrsets that are not declared
        // `rec`.
        node.children().any(|node| {
            (node.kind() == SyntaxKind::NODE_DYNAMIC || node.kind() == SyntaxKind::NODE_STRING)
                && find(name, &node)
        })
    } else {
        // Just search every child in the AST
        node.children().any(|node| find(name, &node))
    }
}
