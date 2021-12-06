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

/// find out if `name` is used in `node`
pub fn find_usage(name: &Ident, node: SyntaxNode<NixLanguage>) -> bool {
    // TODO: return false if shadowed by other let/rec/param binding

    if node.kind() == SyntaxKind::NODE_IDENT {
        Ident::cast(node).expect("Ident::cast").as_str() == name.as_str()
    } else {
        node.children().any(|node| find_usage(name, node))
    }
}
