use crate::{dead_code::DeadCode, scope::Scope};
use rnix::{
    ast::{Inherit, LetIn, HasEntry},
    NixLanguage, SyntaxKind,
};
use rowan::{api::SyntaxNode, ast::AstNode};

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
pub fn edit_dead_code(original: &str, dead: impl Iterator<Item = DeadCode>) -> (String, bool) {
    let mut edits = dead.filter_map(dead_to_edit).collect::<Vec<_>>();
    edits.sort_unstable_by(|e1, e2| {
        if e1.start == e2.start {
            e1.end.cmp(&e2.end)
        } else {
            e1.start.cmp(&e2.start)
        }
    });

    let has_changes = !edits.is_empty();

    let edited = apply_edits(original, edits.iter());

    // remove empty `let in`
    let ast = rnix::Root::parse(&edited);
    let mut let_in_edits = Vec::new();
    remove_empty_scopes(&ast.syntax(), &mut let_in_edits);
    if let_in_edits.is_empty() {
        (edited, has_changes)
    } else {
        (apply_edits(&edited, let_in_edits.iter()), true)
    }
}

fn dead_to_edit(dead_code: DeadCode) -> Option<Edit> {
    let range = dead_code.binding.decl_node.text_range();
    let mut start = usize::from(range.start());
    let mut end = usize::from(range.end());
    let mut replace_node = dead_code.binding.decl_node.clone();
    let mut replacement = None;
    match dead_code.scope {
        Scope::LambdaPattern(pattern, _) => {
            if pattern
                .pat_bind().is_some_and(|at| at.ident().expect("at.ident").syntax() == &dead_code.binding.decl_node)
            {
                if let Some(pattern_bind_node) = pattern
                    .syntax()
                    .children()
                    .find(|child| child.kind() == SyntaxKind::NODE_PAT_BIND)
                {
                    // `dead @ { ... }`, `{ ... } @ dead` forms
                    let pattern_bind_range = pattern_bind_node.text_range();
                    start = usize::from(pattern_bind_range.start());
                    end = usize::from(pattern_bind_range.end());
                    // also remove trailing whitespace for this form
                    if let Some(next) = pattern_bind_node.next_sibling_or_token() {
                        if next.kind() == SyntaxKind::TOKEN_WHITESPACE {
                            end = usize::from(next.text_range().end());
                        }
                    }
                    replacement = Some(String::new());
                    replace_node = pattern_bind_node;
                }
            } else {
                let mut tokens = pattern.syntax().children_with_tokens().skip_while(|node| {
                    node.as_node()
                        .map_or(true, |node| *node != dead_code.binding.decl_node)
                });
                tokens.next();
                for token in tokens {
                    if token.kind() == SyntaxKind::TOKEN_COMMA {
                        // up to the next comma
                        end = usize::from(token.text_range().end());
                        replacement = Some(String::new());
                        break;
                    } else if token.kind() != SyntaxKind::TOKEN_WHITESPACE {
                        // delete only whitespace
                        break;
                    }
                }
            }
        }

        Scope::LambdaArg(name, _) => {
            replacement = Some(format!("_{name}"));
        }

        Scope::LetIn(let_in) => {
            if let_in
                .attrpath_values()
                .any(|entry| *entry.syntax() == dead_code.binding.decl_node)
            {
                replacement = Some(String::new());
            } else if let Some(ident) = let_in
                .inherits()
                .flat_map(|inherit| {
                    inherit
                        .attrs()
                        .filter(|attr| attr.syntax() == &dead_code.binding.decl_node)
                })
                .next()
            {
                let range = ident.syntax().text_range();
                start = usize::from(range.start());
                end = usize::from(range.end());
                replace_node = ident.syntax().clone();
                replacement = Some(String::new());
            }
        }

        Scope::RecAttrSet(_) => {}
    }

    replacement.map(|replacement| {
        // remove whitespace before node
        if let Some(prev) = replace_node.prev_sibling_or_token() {
            if prev.kind() == SyntaxKind::TOKEN_WHITESPACE {
                start = usize::from(prev.text_range().start());
            }
        }

        Edit {
            start,
            end,
            replacement,
        }
    })
}

fn remove_empty_scopes(node: &SyntaxNode<NixLanguage>, edits: &mut Vec<Edit>) {
    match node.kind() {
        // remove empty `let in` constructs
        SyntaxKind::NODE_LET_IN => {
            let let_in = LetIn::cast(node.clone()).expect("LetIn::cast");
            if let_in
                .inherits()
                .all(|inherit| inherit.attrs().next().is_none())
                && let_in.attrpath_values().next().is_none()
            {
                let start = usize::from(node.text_range().start());
                let end = usize::from(let_in.body().expect("let_in.body").syntax().text_range().start());
                edits.push(Edit {
                    start,
                    end,
                    replacement: String::new(),
                });
            }
        }

        // remove empty `inherit;` and `inherit (...);` constructs
        SyntaxKind::NODE_INHERIT => {
            let inherit = Inherit::cast(node.clone()).expect("Inherit::cast");
            if inherit.attrs().next().is_none() {
                let mut start = usize::from(node.text_range().start());
                // remove whitespace before node
                if let Some(prev) = node.prev_sibling_or_token() {
                    if prev.kind() == SyntaxKind::TOKEN_WHITESPACE {
                        start = usize::from(prev.text_range().start());
                    }
                }
                let end = usize::from(node.text_range().end());
                edits.push(Edit {
                    start,
                    end,
                    replacement: String::new(),
                });
            }
        }

        _ => {}
    }

    // recurse through the AST
    for child in node.children() {
        remove_empty_scopes(&child, edits);
    }
}
