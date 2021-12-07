use std::fmt;
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    SyntaxKind,
    types::{
        AttrSet,
        EntryHolder,
        Ident,
        Lambda,
        LetIn,
        Pattern,
        TokenWrapper,
        TypedNode,
    },
};
use crate::binding::Binding;

/// AST subtree that declares variables
#[derive(Debug, Clone)]
pub enum Scope {
    LambdaPattern(Pattern, SyntaxNode<NixLanguage>),
    LambdaArg(Ident, SyntaxNode<NixLanguage>),
    LetIn(LetIn),
    RecAttrSet(AttrSet),
}

impl fmt::Display for Scope {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scope::LambdaPattern(_, _) =>
                write!(fmt, "lambda pattern"),
            Scope::LambdaArg(_, _) =>
                write!(fmt, "lambda argument"),
            Scope::LetIn(_) =>
                write!(fmt, "let binding"),
            Scope::RecAttrSet(_) =>
                write!(fmt, "rec attrset"),
        }
    }
}

impl Scope {
    /// Construct a new Scope *if* this is an AST node that opens a new scope
    pub fn new(node: &SyntaxNode<NixLanguage>) -> Option<Self> {
        match node.kind() {
            SyntaxKind::NODE_LAMBDA => {
                let lambda = Lambda::cast(node.clone())
                    .expect("Lambda::cast");
                let arg = lambda.arg().expect("lambda.arg()");
                let body = lambda.body()
                    .expect("lambda.body()");
                match arg.kind() {
                    SyntaxKind::NODE_IDENT => {
                        let name = Ident::cast(arg.clone())
                            .expect("Ident::cast");
                        Some(Scope::LambdaArg(name, body))
                    }
                    SyntaxKind::NODE_PATTERN => {
                        let pattern = Pattern::cast(arg)
                            .expect("Pattern::cast");
                        Some(Scope::LambdaPattern(pattern, body))
                    }
                    _ => panic!("Unhandled arg kind: {:?}", arg.kind()),
                }
            }

            SyntaxKind::NODE_LET_IN => {
                let let_in = LetIn::cast(node.clone())
                    .expect("LetIn::cast");
                Some(Scope::LetIn(let_in))
            }

            SyntaxKind::NODE_ATTR_SET => {
                let attr_set = AttrSet::cast(node.clone())
                    .expect("AttrSet::cast");
                if attr_set.recursive() {
                    Some(Scope::RecAttrSet(attr_set))
                } else {
                    None
                }
            }

            _ => None
        }
    }

    /// The Bindings this Scope introduces
    pub fn bindings(&self) -> Box<dyn Iterator<Item = Binding>> {
        match self {
            Scope::LambdaPattern(pattern, _) => {
                let mortal = pattern.ellipsis();
                Box::new(
                    pattern.at()
                        .map(|name| {
                            let binding_node = name.node().clone();
                            Binding::new(name, binding_node, true)
                        })
                        .into_iter()
                    .chain(
                        pattern.entries()
                            .map(move |entry| {
                                let name = entry.name()
                                    .expect("entry.name");
                                Binding::new(name, entry.node().clone(), mortal)
                            })
                    )
                )
            }

            Scope::LambdaArg(name, _) => {
                let mortal = ! name.as_str().starts_with("_");
                Box::new(
                    Some(
                        Binding::new(name.clone(), name.node().clone(), mortal)
                    ).into_iter()
                )
            }

            Scope::LetIn(let_in) =>
                Box::new(
                    let_in.inherits()
                        .flat_map(|inherit| {
                            let binding_node = inherit.node().clone();
                            inherit.idents()
                                .map(move |name| {
                                    Binding::new(name, binding_node.clone(), true)
                                })
                        })
                    .chain(
                        let_in.entries()
                            .map(|entry| {
                                let key = entry.key()
                                    .expect("entry.key")
                                    .path().next()
                                    .expect("key.path.next");
                                let name = Ident::cast(key)
                                    .expect("Ident::cast");
                                Binding::new(name, entry.node().clone(), true)
                            })
                    )
                ),

            Scope::RecAttrSet(attr_set) =>
                Box::new(
                    attr_set.inherits()
                        .flat_map(|inherit| {
                            let binding_node = inherit.node().clone();
                            inherit.idents()
                                .map(move |name| {
                                    Binding::new(name, binding_node.clone(), false)
                                })
                        })
                    .chain(
                        attr_set.entries()
                            .map(|entry| {
                                let key = entry.key()
                                    .expect("entry.key")
                                    .path().next()
                                    .expect("key.path.next");
                                let name = Ident::cast(key)
                                    .expect("Ident::cast");
                                Binding::new(name, entry.node().clone(), false)
                            })
                    )
                ),
        }
    }

    /// The code subtrees in which the introduced variables are available
    /// TODO: return &SyntaxNode
    pub fn bodies(&self) -> Box<dyn Iterator<Item = SyntaxNode<NixLanguage>>> {
        match self {
            Scope::LambdaPattern(pattern, body) =>
                Box::new(
                    pattern.entries()
                        .map(|entry| entry.node().clone())
                    .chain(
                        Some(body.clone()).into_iter()
                    )
                ),

            Scope::LambdaArg(_, body) =>
                Box::new(
                    Some(body.clone()).into_iter()
                ),

            Scope::LetIn(let_in) =>
                Box::new(
                    let_in.inherits()
                        .map(|inherit| inherit.node().clone())
                        .chain(
                            let_in.entries()
                                .map(|entry| entry.node().clone())
                        )
                        .chain(let_in.body())
                ),

            Scope::RecAttrSet(attr_set) =>
                Box::new(
                    attr_set.inherits()
                        .map(|inherit| inherit.node().clone())
                        .chain(
                            attr_set.entries()
                                .map(|entry| entry.node().clone())
                        )
                ),
        }
    }
}
