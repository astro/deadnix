#![cfg(test)]

use crate::dead_code::{DeadCode, Settings};
use rnix::types::TokenWrapper;

fn run(content: &str) -> Vec<DeadCode> {
    let ast = rnix::parse(content);
    assert_eq!(0, ast.errors().len());

    Settings {
        no_lambda_arg: false,
        no_underscore: false,
    }
    .find_dead_code(&ast.node())
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
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn let_in_dead_recursive() {
    let results = run("let dead = dead; in false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn let_in_shadowed() {
    let results = run("let dead = true; in let dead = false; in dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
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
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn let_in_inherit_shadowed() {
    let results = run("let inherit (dead) dead; in let inherit (alive) dead; in dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
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
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn lambda_arg_shadowed() {
    let results = run("dead: dead: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
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
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn lambda_at_shadowed() {
    let results = run("dead@{ ... }: dead@{ ... }: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn lambda_pattern_alive() {
    let results = run("{ alive, ... }: alive");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_pattern_dead() {
    let results = run("alive@{ dead, ... }: alive");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn lambda_pattern_no_ellipsis() {
    let results = run("{ alive }: false");
    assert_eq!(0, results.len());
}

#[test]
fn lambda_pattern_shadowed() {
    let results = run("{ dead, ... }: { dead, ... }: dead");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
}

#[test]
fn looped() {
    let results = run("let dead1 = dead2; dead2 = {}; in false");
    assert_eq!(2, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead1");
    assert_eq!(results[1].binding.name.as_str(), "dead2");
}

#[test]
fn rec_attrset_shadowed() {
    let results = run("let dead = false; in rec { dead = true; alive = dead; }");
    assert_eq!(1, results.len());
    assert_eq!(results[0].binding.name.as_str(), "dead");
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
