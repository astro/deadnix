use rnix::{
    types::{Ident, TokenWrapper},
    NixLanguage,
};
use rowan::api::SyntaxNode;

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Ident,
    pub node: SyntaxNode<NixLanguage>,
    mortal: bool,
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.name.as_str() == other.name.as_str()
    }
}
impl Eq for Binding {}

impl Binding {
    pub fn new(name: Ident, node: SyntaxNode<NixLanguage>, mortal: bool) -> Self {
        Binding { name, node, mortal }
    }

    pub fn is_mortal(&self) -> bool {
        self.mortal
    }
}
