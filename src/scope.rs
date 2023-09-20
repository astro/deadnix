use crate::{binding::Binding, usage};
use ariadne::Color;
use rnix::{
    ast::{AttrSet, Ident, Lambda, LetIn, Pattern, Param, HasEntry, Attr},
    NixLanguage, SyntaxKind,
};
use rowan::{api::SyntaxNode, ast::AstNode};
use std::fmt;

/// AST subtree that declares variables
#[derive(Debug, Clone)]
pub enum Scope {
    /// `{ ... }: ...`
    LambdaPattern(Pattern, SyntaxNode<NixLanguage>),
    /// `...: ...`
    LambdaArg(Ident, SyntaxNode<NixLanguage>),
    /// `let ... in ...`
    LetIn(LetIn),
    /// `rec { ... }`
    RecAttrSet(AttrSet),
}

impl fmt::Display for Scope {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scope::LambdaPattern(_, _) => write!(fmt, "lambda pattern"),
            Scope::LambdaArg(_, _) => write!(fmt, "lambda argument"),
            Scope::LetIn(_) => write!(fmt, "let binding"),
            Scope::RecAttrSet(_) => write!(fmt, "rec attrset"),
        }
    }
}

impl Scope {
    /// Construct a new Scope *if* this is an AST node that opens a new scope
    pub fn new(node: &SyntaxNode<NixLanguage>) -> Option<Self> {
        match node.kind() {
            SyntaxKind::NODE_LAMBDA => {
                let lambda = Lambda::cast(node.clone()).expect("Lambda::cast");
                let param = lambda.param().expect("lambda.param()");
                let body = lambda.body().expect("lambda.body()").syntax().clone();
                match param {
                    Param::IdentParam(ident_param) => {
                        let name = ident_param.ident().expect("IdentParam.ident()");
                        Some(Scope::LambdaArg(name, body))
                    }
                    Param::Pattern(pattern) => {
                        Some(Scope::LambdaPattern(pattern, body))
                    }
                }
            }

            SyntaxKind::NODE_LET_IN => {
                let let_in = LetIn::cast(node.clone()).expect("LetIn::cast");
                Some(Scope::LetIn(let_in))
            }

            SyntaxKind::NODE_ATTR_SET => {
                let attr_set = AttrSet::cast(node.clone()).expect("AttrSet::cast");
                if attr_set.rec_token().is_some() {
                    Some(Scope::RecAttrSet(attr_set))
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    /// Is this a `...: ...` which can be ignored in
    /// [`Settings`](`crate::Settings::no_lambda_arg`)
    pub fn is_lambda_arg(&self) -> bool {
        matches!(self, Scope::LambdaArg(_, _))
    }

    /// Is this a `{ ... }: ...` which can be ignored in
    /// [`Settings`](`crate::Settings::no_lambda_pattern_names`)
    pub fn is_lambda_pattern_name(&self, name: &Ident) -> bool {
        if let Scope::LambdaPattern(pattern, _) = self {
            pattern
                .pat_entries()
                .any(|entry| entry.ident().expect("entry.ident").syntax().text() == name.syntax().text())
        } else {
            false
        }
    }

    /// The set of [`Binding`]s this [`Scope`] introduces
    pub fn bindings(&self) -> Box<dyn Iterator<Item = Binding>> {
        match self {
            Scope::LambdaPattern(pattern, _) => Box::new(
                pattern
                    .pat_bind()
                    .and_then(|name| name.ident())
                    .map(|name| Binding::new(name.clone(), name.syntax().clone(), true))
                    .into_iter()
                    .chain(pattern.pat_entries().map(move |entry| {
                        let name = entry.ident().expect("entry.ident");
                        Binding::new(name, entry.syntax().clone(), true)
                    })),
            ),

            Scope::LambdaArg(name, _) => {
                let mortal = name.syntax().text().char_at(0.into()) != Some('_');
                Box::new(
                    Some(Binding::new(
                        name.clone(),
                        name.syntax().clone(),
                        mortal,
                    ))
                    .into_iter(),
                )
            }

            Scope::LetIn(let_in) => Box::new(
                let_in
                    .inherits()
                    .flat_map(|inherit| {
                        inherit.attrs().filter_map(move |attr|
                            match attr {
                                Attr::Ident(ident) => Some(ident),
                                _ => None,
                            }
                        ).map(move |ident| {
                            Binding::new(
                                ident.clone(),
                                ident.syntax().clone(),
                                true
                            )
                        })
                    })
                    .chain(let_in.attrpath_values().filter_map(|entry| {
                        let attrpath = entry.attrpath()
                            .expect("entry.attrpath");
                        match attrpath.attrs().next() {
                            Some(Attr::Ident(name)) =>
                                Some(
                                    Binding::new(name, entry.syntax().clone(), true)
                                ),
                            _ => None,
                        }
                    })),
            ),

            Scope::RecAttrSet(attr_set) => Box::new(
                attr_set
                    .inherits()
                    .flat_map(|inherit| {
                        inherit.attrs().filter_map(move |attr| {
                            match attr {
                                Attr::Ident(ref name) =>
                                    Some(
                                        Binding::new(name.clone(), attr.syntax().clone(), false)
                                    ),
                                _ =>
                                    None,
                            }
                        })
                    })
                    .chain(attr_set.attrpath_values().filter_map(|entry| {
                        let key = entry
                            .attrpath()
                            .expect("entry.attrpath")
                            .attrs()
                            .next();
                        match key {
                            Some(Attr::Ident(name)) =>
                                Some(
                                    Binding::new(name, entry.syntax().clone(), false)
                                ),
                            _ => None,
                        }
                    })),
            ),
        }
    }

    /// The code subtrees in which the introduced variables are available
    pub fn bodies(&self) -> Box<dyn Iterator<Item = SyntaxNode<NixLanguage>>> {
        match self {
            Scope::LambdaPattern(pattern, body) => Box::new(
                pattern
                    .pat_entries()
                    .map(|entry| entry.syntax().clone())
                    .chain(Some(body.clone())),
            ),

            Scope::LambdaArg(_, body) => Box::new(Some(body.clone()).into_iter()),

            Scope::LetIn(let_in) => Box::new(
                let_in
                    .inherits()
                    .filter_map(|inherit| inherit.from().map(|from| from.syntax().clone()))
                    .chain(
                        let_in.attrpath_values().map(|entry| entry.syntax().clone())
                    )
                    .chain(
                        let_in.body().map(|body| body.syntax().clone())
                    ),
            ),

            Scope::RecAttrSet(attr_set) => Box::new(
                attr_set
                    .inherits()
                    .map(|inherit| inherit.syntax().clone())
                    .chain(attr_set.attrpath_values().map(|entry| entry.syntax().clone())),
            ),
        }
    }

    /// Check the `inherit (var) ...` and `inherit vars` clauses for a
    /// given `name`.
    ///
    /// Although a scope may shadow existing variable bindings, it can
    /// `inherit` bindings from the outer scope.
    pub fn inherits_from(&self, name: &Ident) -> bool {
        match self {
            Scope::LambdaPattern(_, _) | Scope::LambdaArg(_, _) => false,

            Scope::LetIn(let_in) => let_in.inherits().any(|inherit| {
                inherit.from().map_or_else(
                    || usage::find(name, inherit.syntax()),
                    |from| usage::find(name, from.syntax()),
                )
            }),

            Scope::RecAttrSet(attr_set) => attr_set.inherits().any(|inherit| {
                inherit.from().map_or_else(
                    || usage::find(name, inherit.syntax()),
                    |from| usage::find(name, from.syntax()),
                )
            }),
        }
    }

    /// Output color for dead code warnings
    pub fn color(&self) -> Color {
        match self {
            Scope::LambdaPattern(_, _) => Color::Magenta,
            Scope::LambdaArg(_, _) => Color::Cyan,
            Scope::LetIn(_) => Color::Red,
            Scope::RecAttrSet(_) => Color::Yellow,
        }
    }
}
