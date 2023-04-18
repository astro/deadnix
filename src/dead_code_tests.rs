#![cfg(test)]

use rowan::ast::AstNode;
use crate::dead_code::{DeadCode, Settings};

fn run(content: &str) -> Vec<DeadCode> {
    let ast = rnix::Root::parse(content);
    assert_eq!(0, ast.errors().len());

    Settings {
        no_lambda_arg: false,
        no_lambda_pattern_names: false,
        no_underscore: false,
    }
    .find_dead_code(&ast.syntax())
}

#[test]
fn let_in_alive() {
    let results = run("let alive = 23; in alive");
    assert_eq!(0, results.len());
}

#[test]
fn let_in_alive_deep() {
    let results = run("let alive = 23; in if true then 42 else { ... }: alive");
    assert_eq!(0, results.len());
}

#[test]
fn let_in_dead() {
    let results = run("let dead = 23; in false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn let_in_dead_multi() {
    let results = run("let dead1 = 23; dead2 = 5; dead3 = 42; in false");
    assert_eq!(3, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead1");
    assert_eq!(results[1].binding.name.to_string(), "dead2");
    assert_eq!(results[2].binding.name.to_string(), "dead3");
}

#[test]
fn let_in_dead_multi_recursive() {
    let results = run("let dead1 = dead2; dead2 = dead3; dead3 = 42; in false");
    assert_eq!(3, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead1");
    assert_eq!(results[1].binding.name.to_string(), "dead2");
    assert_eq!(results[2].binding.name.to_string(), "dead3");
}

#[test]
fn let_in_dead_recursive() {
    let results = run("let dead = dead; in false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn let_in_shadowed() {
    let results = run("let dead = true; in let dead = false; in dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn let_in_inherit_alive() {
    let results = run("let alive = {}; inherit (alive) key; in key");
    assert_eq!(0, results.len());
}

#[test]
fn let_in_inherit_dead() {
    let results = run("let inherit (alive) dead; in alive");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn let_in_inherit_dead_multi() {
    let results = run("let inherit (grave) dead1 dead2 dead3; in false");
    assert_eq!(3, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead1");
    assert_eq!(results[1].binding.name.to_string(), "dead2");
    assert_eq!(results[2].binding.name.to_string(), "dead3");
}

#[test]
fn let_in_inherit_dead_recursive_multi() {
    let results =
        run("let inherit (grave) dead1; inherit (dead1) dead2; inherit (dead2) dead3; in false");
    assert_eq!(3, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead1");
    assert_eq!(results[1].binding.name.to_string(), "dead2");
    assert_eq!(results[2].binding.name.to_string(), "dead3");
}

#[test]
fn let_in_inherit_shadowed() {
    let results = run("let inherit (dead) dead; in let inherit (alive) dead; in dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_arg_alive() {
    let results = run("alive: alive");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_arg_underscore() {
    let results = run("_unused: alive");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_arg_dead() {
    let results = run("dead: false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_arg_shadowed() {
    let results = run("dead: dead: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_at_alive() {
    let results = run("alive@{ ... }: alive");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_at_pattern_alive() {
    let results = run("alive@{ x ? alive, ... }: x");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_at_dead() {
    let results = run("dead@{ ... }: false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_at_shadowed() {
    let results = run("dead@{ ... }: dead@{ ... }: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_pattern_alive() {
    let results = run("{ alive, ... }: alive");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_pattern_dead_ellipsis_alias() {
    let results = run("alive@{ dead, ... }: alive");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_pattern_dead_simple() {
    let results = run("{ dead }: false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_pattern_alias() {
    let results = run("{ dead }@args: args");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn lambda_pattern_shadowed() {
    let results = run("{ dead, ... }: { dead, ... }: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn looped() {
    let results = run("let dead1 = dead2; dead2 = {}; in false");
    assert_eq!(2, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead1");
    assert_eq!(results[1].binding.name.to_string(), "dead2");
}

#[test]
fn rec_attrset_shadowed() {
    let results = run("let dead = false; in rec { dead = true; alive = dead; }");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn let_inherit_in_let_inherit_alive() {
    let results = run("let alive = true; in let inherit alive; in alive");
    assert_eq!(0, results.len());
}

#[test]
fn let_inherit_in_rec_attrset_alive() {
    let results = run("let alive = true; in rec { inherit alive; }");
    assert_eq!(0, results.len());
}

#[test]
fn skip() {
    let results = run("
# deadnix: skip
let dead = 0;
in alive
    ");
    assert_eq!(0, results.len());
}

#[test]
fn skip_no_multiline() {
    let results = run("
# deadnix: skip
let dead1 = 0;
    dead2 = 1;
in alive
    ");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead2");
}

#[test]
fn skip_no_comment() {
    let results = run("
# deadnix: skip
# ignore the above statement
let dead = 1;
in alive
    ");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead");
}

#[test]
fn skip_attrset() {
    let results = run("
# deadnix: skip
{ dead1
, dead2
}:
alive
    ");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead2");
}

#[test]
fn skip_lambda_arg() {
    let results = run("
# deadnix: skip
dead1:
dead2:
alive
    ");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "dead2");
}

#[test]
fn shadowed_by_skip() {
    let nix = "
let
  shadowed = 0;
in let
# deadnix: skip
  shadowed = 1;
in shadowed
    ";
    let results = run(nix);
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.to_string(), "shadowed");
    let first_pos = nix.find("shadowed").unwrap();
    assert_eq!(usize::from(results[0].binding.name.syntax().text_range().start()), first_pos);
}
