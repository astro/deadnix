use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    types::{
        Ident,
        TokenWrapper,
    },
};

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Ident,
    pub node: SyntaxNode<NixLanguage>,
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node &&
        self.name.as_str() == other.name.as_str()
    }
}
impl Eq for Binding {}

impl Binding {
    pub fn new(name: Ident, node: SyntaxNode<NixLanguage>) -> Self {
        Binding { name, node }
    }
}
