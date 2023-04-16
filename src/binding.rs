use rnix::{
    ast::Ident,
    NixLanguage,
};
use rowan::api::SyntaxNode;

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Ident,
    pub body_node: SyntaxNode<NixLanguage>,
    pub decl_node: SyntaxNode<NixLanguage>,
    mortal: bool,
}

impl Binding {
    pub fn new(
        name: Ident,
        body_node: SyntaxNode<NixLanguage>,
        decl_node: SyntaxNode<NixLanguage>,
        mortal: bool,
    ) -> Self {
        Binding {
            name,
            body_node,
            decl_node,
            mortal,
        }
    }

    pub fn is_mortal(&self) -> bool {
        self.mortal
    }
}
