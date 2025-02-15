use rnix::{ast::Ident, NixLanguage, SyntaxKind};
use rowan::{api::SyntaxNode, ast::AstNode};

/// This string in a Nix comment above an unused declaration shall
/// force us to skip it.
///
/// ```nix
/// let
///   # deadnix: skip
///   skeletonsInTheBasement =
/// ```
const PRAGMA_SKIP: &str = "deadnix: skip";

/// A Nix variable binding
#[derive(Debug, Clone)]
pub struct Binding {
    /// Variable name
    pub name: Ident,
    /// Syntax node of declaration itself
    pub decl_node: SyntaxNode<NixLanguage>,
    mortal: bool,
}

impl Binding {
    /// Create a new Binding
    pub fn new(name: Ident, decl_node: SyntaxNode<NixLanguage>, mortal: bool) -> Self {
        Binding {
            name,
            decl_node,
            mortal,
        }
    }

    /// Can die?
    ///
    /// Not mortal are `rec { ... }`, and lambda args that already
    /// start with `_`.
    pub fn is_mortal(&self) -> bool {
        self.mortal
    }

    /// Does the name start with `_`, signifying an anonymous
    /// variable?
    pub fn starts_with_underscore(&self) -> bool {
        self.name.syntax().text().char_at(0.into()) == Some('_')
    }

    /// Searches through tokens backwards for `PRAGMA_SKIP` until at
    /// least two linebreaks are seen
    pub fn has_pragma_skip(&self) -> bool {
        let mut line_breaks = 0;
        let mut token = self.decl_node.first_token().unwrap();
        while let Some(prev) = token.prev_token() {
            token = prev;

            match token.kind() {
                SyntaxKind::TOKEN_WHITESPACE => {
                    line_breaks += token.text().matches('\n').count();
                    if line_breaks > 1 {
                        break;
                    }
                }

                SyntaxKind::TOKEN_COMMENT if token.text().contains(PRAGMA_SKIP) => return true,

                _ => {}
            }
        }

        // No PRAGMA_SKIP found
        false
    }
}
