#![cfg(test)]

use crate::dead_code::Settings;

fn run(content: &str) -> (String, bool) {
    let ast = rnix::parse(content);
    assert_eq!(0, ast.errors().len());

    let results = Settings {
        no_lambda_arg: false,
        no_lambda_pattern_names: false,
        no_underscore: false,
    }
    .find_dead_code(&ast.node());
    crate::edit::edit_dead_code(content, results.into_iter())
}

macro_rules! no_edits {
    ($s: expr) => {
        let s = $s.to_string();
        assert_eq!(run(&s), (s, false));
    };
}

macro_rules! has_edits {
    ($s1: expr, $s2: expr) => {
        let s1 = $s1.to_string();
        let s2 = $s2.to_string();
        assert_eq!(run(&s1), (s2, true));
    };
}

#[test]
fn let_in_alive() {
    no_edits!("let alive = 23; in alive");
}

#[test]
fn let_in_alive_deep() {
    no_edits!("let alive = 23; in if true then 42 else { ... }: alive");
}

#[test]
fn let_in_alive_dead() {
    has_edits!(
        "let alive = 42; dead = 23; in alive",
        "let alive = 42; in alive"
    );
}

#[test]
fn let_in_dead_only() {
    has_edits!(
        "let dead = 42; in alive",
        "alive"
    );
}

#[test]
fn let_inherit_in_alive() {
    no_edits!("let inherit (x) alive; in alive");
}

#[test]
fn let_inherit_in_alive_dead() {
    has_edits!(
        "let inherit alive dead; in alive",
        "let inherit alive; in alive"
    );
}

#[test]
fn let_inherit_dead_let_alive_in_dead() {
    has_edits!(
        "let inherit dead; alive = true; in alive",
        "let alive = true; in alive"
    );
}

#[test]
fn let_inherit_in_dead_only() {
    has_edits!(
        "let inherit dead; in alive",
        "alive"
    );
}

#[test]
fn let_inherit_multi_in_dead_only() {
    has_edits!(
        "let inherit dead1 dead2 dead3; in alive",
        "alive"
    );
}

/// <https://github.com/astro/deadnix/issues/7>
#[test]
fn let_dead_only_whitespacing() {
    has_edits!(
        "{ used }: let unused = {}; in used",
        "{ used }: used"
    );
}

#[test]
fn let_inherit_from_in_alive() {
    no_edits!("let inherit (x) alive; in alive");
}

#[test]
fn let_inherit_from_in_alive_dead() {
    has_edits!(
        "let inherit (x) alive dead; in alive",
        "let inherit (x) alive; in alive"
    );
}

#[test]
fn let_inherit_from_dead_let_alive_in_dead() {
    has_edits!(
        "let inherit (x) dead; alive = true; in alive",
        "let alive = true; in alive"
    );
}

#[test]
fn let_inherit_from_in_dead_only() {
    has_edits!(
        "let inherit (x) dead; in alive",
        "alive"
    );
}

#[test]
fn let_inherit_from_multi_in_dead_only() {
    has_edits!(
        "let inherit (grave) dead1 dead2 dead3; in alive",
        "alive"
    );
}

#[test]
fn lambda_arg_alive() {
    no_edits!("alive: alive");
}

#[test]
fn lambda_arg_dead() {
    has_edits!(
        "dead: false",
        "_dead: false"
    );
}

#[test]
fn lambda_arg_anon() {
    no_edits!("_anon: false");
}

#[test]
fn lambda_at_pattern_dead() {
    has_edits!(
        "dead@{ dead2 ? dead, ... }: false",
        "{ ... }: false"
    );
}

#[test]
fn lambda_lead_at_dead() {
    has_edits!(
        "dead@{ ... }: false",
        "{ ... }: false"
    );
}

#[test]
fn lambda_trail_at_dead() {
    has_edits!(
        "{ ... }@dead: false",
        "{ ... }: false"
    );
}

#[test]
fn lambda_lead_at_space_dead() {
    has_edits!(
        "dead @ { ... }: false",
        "{ ... }: false"
    );
}

#[test]
fn lambda_trail_at_space_dead() {
    has_edits!(
        "{ ... } @ dead: false",
        "{ ... }: false"
    );
}

#[test]
fn lambda_at_shadowed() {
    has_edits!(
        "dead@{ ... }: dead@{ ... }: dead",
        "{ ... }: dead@{ ... }: dead"
    );
}

#[test]
fn lambda_pattern_dead() {
    has_edits!(
        "alive@{ dead, ... }: alive",
        "alive@{ ... }: alive"
    );
}

#[test]
fn lambda_pattern_default_dead() {
    has_edits!(
        "alive@{ dead ? true, ... }: alive",
        "alive@{ ... }: alive"
    );
}

#[test]
fn lambda_pattern_mixed() {
    has_edits!(
        "dead1@{ dead2, alive, ... }: alive",
        "{ alive, ... }: alive"
    );
}

#[test]
fn lambda_pattern_dead_multiline() {
    has_edits!(
        "{ alive\n, dead\n, ... }:\nalive",
        "{ alive\n, ... }:\nalive"
    );
}
