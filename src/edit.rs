use std::collections::HashMap;
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    SyntaxKind,
    types::{
        EntryHolder,
        Inherit,
        LetIn,
        TokenWrapper, TypedNode
    },
};
use crate::{
    dead_code::DeadCode,
    scope::Scope,
};


#[derive(Debug)]
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
}

fn apply_edits<'a>(src: &str, edits: impl Iterator<Item = &'a Edit>) -> String {
    let mut pos = 0;
    let mut result = String::with_capacity(src.len());
    for edit in edits {
        if pos <= edit.end {
            result.push_str(&src[pos..edit.start]);
            result.push_str(&edit.replacement);
            pos = edit.end;
        }
    }
    result.push_str(&src[pos..]);
    result
}

/// Deletes `nodes` from content
///
/// assumes `node` to be presorted
pub fn edit_dead_code(original: &str, node: SyntaxNode<NixLanguage>, dead: impl Iterator<Item = DeadCode>) -> String {
    let mut dead = dead.map(|result| (result.binding.node.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut edits = Vec::with_capacity(dead.len());
    scan(node, &mut dead, &mut edits);

    edits.sort_unstable_by(|e1, e2| {
        if e1.start == e2.start {
            e1.end.cmp(&e2.end)
        } else {
            e1.start.cmp(&e2.start)
        }
    });

    let edited = apply_edits(original, edits.iter());

    // remove empty `let in`
    let ast = rnix::parse(&edited);
    let mut let_in_edits = Vec::new();
    remove_empty_scopes(ast.node(), &mut let_in_edits);
    if edits.len() > 0 {
        apply_edits(&edited, let_in_edits.iter())
    } else {
        edited
    }
}

fn scan(node: SyntaxNode<NixLanguage>, dead: &mut HashMap<SyntaxNode<NixLanguage>, DeadCode>, edits: &mut Vec<Edit>) {
    if let Some(dead_code) = dead.remove(&node) {
        let range = dead_code.binding.node.text_range();
        let mut start = usize::from(range.start());
        let mut end = usize::from(range.end());
        let mut replace_node = node.clone();
        let mut replacement = None;
        match dead_code.scope {
            Scope::LambdaPattern(pattern, _) => {
                if pattern.at().map(|at| *at.node() == node).unwrap_or(false) {
                    if let Some(next) = node.next_sibling_or_token() {
                        // dead@{ ... } form
                        if next.kind() == SyntaxKind::TOKEN_AT {
                            end = usize::from(next.text_range().end());
                            replacement = Some("".to_string());
                        }
                    }
                    if replacement.is_none() {
                        if let Some(prev) = node.prev_sibling_or_token() {
                            // { ... }@dead form
                            if prev.kind() == SyntaxKind::TOKEN_AT {
                                start = usize::from(prev.text_range().start());
                                replacement = Some("".to_string());
                            }
                        }
                    }
                } else {
                    if let Some(next) = node.next_sibling_or_token() {
                        // { dead, ... } form
                        if next.kind() == SyntaxKind::TOKEN_COMMA {
                            end = usize::from(next.text_range().end());
                            replacement = Some("".to_string());
                        }
                    }
                }
            }

            Scope::LambdaArg(_, _) => {
                replacement = Some("_".to_string());
            }

            Scope::LetIn(let_in) => {
                if let_in.entries().any(|entry|
                    *entry.node() == node
                ) {
                    replacement = Some("".to_string());
                } else if let Some(inherit) = let_in.inherits().find(|inherit|
                    *inherit.node() == node
                ) {
                    if let Some(ident) = inherit.idents().find(|ident|
                        ident.as_str() == dead_code.binding.name.as_str()
                    ) {
                        let range = ident.node().text_range();
                        start = usize::from(range.start());
                        end = usize::from(range.end());
                        replace_node = ident.node().clone();
                        replacement = Some("".to_string());
                    }
                }
            }

            Scope::RecAttrSet(_) => {}
        }

        if let Some(replacement) = replacement {
            // remove whitespace before node
            if let Some(prev) = replace_node.prev_sibling_or_token() {
                if prev.kind() == SyntaxKind::TOKEN_WHITESPACE {
                    start = usize::from(prev.text_range().start());
                }
            }

            edits.push(Edit {
                start, end,
                replacement,
            });
        }
    }

    // recurse through the AST
    for child in node.children() {
        scan(child, dead, edits);
    }
}

fn remove_empty_scopes(node: SyntaxNode<NixLanguage>, edits: &mut Vec<Edit>) {
    match node.kind() {
        // remove empty `let in` constructs
        SyntaxKind::NODE_LET_IN => {
            let let_in = LetIn::cast(node.clone())
                .expect("LetIn::cast");
            if let_in.inherits().all(|inherit| inherit.idents().next().is_none())
                && let_in.entries().next().is_none() {
                    let mut start = usize::from(node.text_range().start());
                    // remove whitespace before node
                    if let Some(prev) = node.prev_sibling_or_token() {
                        if prev.kind() == SyntaxKind::TOKEN_WHITESPACE {
                            start = usize::from(prev.text_range().start());
                        }
                    }
                    let end = usize::from(let_in.body().expect("let_in.body").text_range().start());
                    edits.push(Edit {
                        start, end,
                        replacement: "".to_string(),
                    });
                }
        }

        // remove empty `inherit;` and `inherit (...);` constructs
        SyntaxKind::NODE_INHERIT => {
            let inherit = Inherit::cast(node.clone())
                .expect("Inherit::cast");
            if inherit.idents().next().is_none() {
                    let mut start = usize::from(node.text_range().start());
                    // remove whitespace before node
                    if let Some(prev) = node.prev_sibling_or_token() {
                        if prev.kind() == SyntaxKind::TOKEN_WHITESPACE {
                            start = usize::from(prev.text_range().start());
                        }
                    }
                    let end = usize::from(node.text_range().end());
                    edits.push(Edit {
                        start, end,
                        replacement: "".to_string(),
                    });
            }
        }

        _ => {}
    }

    // recurse through the AST
    for child in node.children() {
        remove_empty_scopes(child, edits);
    }
}
