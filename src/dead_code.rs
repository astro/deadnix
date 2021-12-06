use std::fmt;
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    SyntaxKind,
    types::{
        EntryHolder, Ident, Lambda, LetIn,
        Pattern,
        TokenWrapper,
        TypedNode,
    },
};
use crate::usage::find_usage;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BindingKind {
    LambdaAt,
    LambdaPattern,
    LambdaArg,
    LetInEntry,
    LetInInherit,
}

impl fmt::Display for BindingKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BindingKind::LambdaAt =>
                write!(fmt, "lambda @-binding"),
            BindingKind::LambdaPattern =>
                write!(fmt, "lambda pattern"),
            BindingKind::LambdaArg =>
                write!(fmt, "lambda argument"),
            BindingKind::LetInEntry =>
                write!(fmt, "let in binding"),
            BindingKind::LetInInherit =>
                write!(fmt, "let in inherit binding"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadCode {
    pub kind: BindingKind,
    pub name: Ident,
    pub node: SyntaxNode<NixLanguage>,
}

impl fmt::Display for DeadCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} {}", self.kind, self.name.as_str())
    }
}

pub fn find_dead_code(node: SyntaxNode<NixLanguage>) -> Vec<DeadCode> {
    let mut results = Vec::new();
    scan(node, &mut results);
    results
}

// recursively scan the AST, accumulating results
fn scan(node: SyntaxNode<NixLanguage>, results: &mut Vec<DeadCode>) {
    match node.kind() {
        SyntaxKind::NODE_LAMBDA => {
            let lambda = Lambda::cast(node.clone())
                .expect("Lambda::cast");
            if let Some(arg) = lambda.arg() {
                match arg.kind() {
                    SyntaxKind::NODE_IDENT => {
                        let name = Ident::cast(arg.clone())
                            .expect("Ident::cast");
                        if !find_usage(&name, lambda.body().expect("lambda.body()")) {
                            results.push(DeadCode {
                                kind: BindingKind::LambdaArg,
                                name,
                                node: arg,
                            });
                        }
                    }
                    SyntaxKind::NODE_PATTERN => {
                        let pattern = Pattern::cast(arg)
                            .expect("Pattern::cast");
                        if let Some(name) = pattern.at() {
                            // check if used in the pattern bindings, or the body
                            if !pattern.entries().any(|entry| find_usage(&name, entry.node().clone()))
                            && !find_usage(&name, lambda.body().expect("body"))
                            {
                                results.push(DeadCode {
                                    kind: BindingKind::LambdaAt,
                                    node: name.node().clone(),
                                    name,
                                });
                            }
                        }
                        if pattern.ellipsis() {
                            // `...` means args can be dropped
                            for entry in pattern.entries() {
                                let name = entry.name()
                                    .expect("entry.name()");
                                // check if used in the other pattern bindings, or the body
                                if !pattern.entries().any(|entry| {
                                    let other_name = entry.name().expect("entry.name()");
                                    other_name.as_str() != name.as_str() &&
                                    find_usage(&name, entry.node().clone())
                                })
                                && !find_usage(&name, lambda.body().expect("lambda.body()")) {
                                    results.push(DeadCode {
                                        kind: BindingKind::LambdaPattern,
                                        node: name.node().clone(),
                                        name,
                                    });
                                }
                            }
                        }
                    }
                    _ => panic!("Unhandled arg kind: {:?}", arg.kind()),
                }
            }
        }

        SyntaxKind::NODE_LET_IN => {
            let let_in = LetIn::cast(node.clone())
                .expect("LetIn::cast");
            if let Some(body) = let_in.body() {
                for key_value in let_in.entries() {
                    let key = key_value.key()
                        .expect("key_value.key()");
                    let name_node = key.path().next()
                        .expect("key.path()");
                    let name = Ident::cast(name_node.clone())
                            .expect("Ident::cast");
                    if !let_in.entries().any(|entry| {
                        let other_name = entry.key().expect("entry.key()")
                            .path().next().expect("path().next()");
                        let other_name = Ident::cast(other_name)
                            .expect("Ident::cast");
                        other_name.as_str() != name.as_str() &&
                        find_usage(&name, entry.node().clone())
                    })
                    && !let_in.inherits().any(|inherit|
                        inherit.from().map(|from|
                            find_usage(&name, from.node().clone())
                        ).unwrap_or(false))
                    && !find_usage(&name, body.clone()) {
                        results.push(DeadCode {
                            kind: BindingKind::LetInEntry,
                            node: name_node,
                            name,
                        });
                    }
                }
                for inherit in let_in.inherits() {
                    for ident in inherit.idents() {
                        let name_node = ident.node();
                        let name = Ident::cast(name_node.clone())
                            .expect("Ident::cast");
                        if !let_in.entries().any(|entry| find_usage(&name, entry.node().clone()))
                        && !let_in.inherits().any(|inherit|
                            inherit.from().map(|from|
                                find_usage(&name, from.node().clone())
                            ).unwrap_or(false))
                        && !find_usage(&name, body.clone()) {
                            results.push(DeadCode {
                                kind: BindingKind::LetInInherit,
                                node: name_node.clone(),
                                name,
                            });
                        }
                    }
                }
            }
        }

        _ => {}
    }

    for child in node.children() {
        scan(child, results);
    }
}
