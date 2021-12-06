#![cfg(test)]

use rnix::types::TokenWrapper;
use crate::dead_code::{BindingKind, DeadCode, find_dead_code};

fn run(content: &str) -> Vec<DeadCode> {
    let ast = rnix::parse(&content);
    assert_eq!(0, ast.errors().len());

    find_dead_code(ast.node())
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
    assert_eq!(results[0].kind, BindingKind::LetInEntry);
    assert_eq!(results[0].name.as_str(), "dead");
}

#[test]
fn let_in_dead_recursive() {
    let results = run("let dead = dead; in false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].kind, BindingKind::LetInEntry);
    assert_eq!(results[0].name.as_str(), "dead");
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
    assert_eq!(results[0].kind, BindingKind::LetInInherit);
    assert_eq!(results[0].name.as_str(), "dead");
}

#[test]
fn lambda_arg_dead() {
    let results = run("dead: false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].kind, BindingKind::LambdaArg);
    assert_eq!(results[0].name.as_str(), "dead");
}

#[test]
fn lambda_at_dead() {
    let results = run("dead@{ ... }: false");
    assert_eq!(1, results.len());
    assert_eq!(results[0].kind, BindingKind::LambdaAt);
    assert_eq!(results[0].name.as_str(), "dead");
}

#[test]
fn lambda_pattern_dead() {
    let results = run("alive@{ dead, ... }: alive");
    assert_eq!(1, results.len());
    assert_eq!(results[0].kind, BindingKind::LambdaPattern);
    assert_eq!(results[0].name.as_str(), "dead");
}
