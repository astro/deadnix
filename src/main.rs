use std::{env::args, fmt, fs};
use rowan::api::SyntaxNode;
use rnix::{
    NixLanguage,
    SyntaxKind,
    types::{
        AttrSet,
        EntryHolder, Ident, Lambda, LetIn,
        Pattern,
        TokenWrapper,
        TypedNode,
    },
};

enum ResultKind {
    LambdaAt,
    LambdaPattern,
    LambdaArg,
    LetInEntry,
    LetInInherit,
}

impl fmt::Display for ResultKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultKind::LambdaAt =>
                write!(fmt, "lambda @-binding"),
            ResultKind::LambdaPattern =>
                write!(fmt, "lambda pattern"),
            ResultKind::LambdaArg =>
                write!(fmt, "lambda argument"),
            ResultKind::LetInEntry =>
                write!(fmt, "let in binding"),
            ResultKind::LetInInherit =>
                write!(fmt, "let in inherit binding"),
        }
    }
}

struct ResultItem {
    kind: ResultKind,
    name: Ident,
    node: SyntaxNode<NixLanguage>,
}

/// find out if `name` is used in `node`
fn find_usage(name: &Ident, node: SyntaxNode<NixLanguage>) -> bool {
    // TODO: return false if shadowed by other let/rec/param binding

    if node.kind() == SyntaxKind::NODE_IDENT {
        Ident::cast(node).expect("Ident::cast").as_str() == name.as_str()
    } else {
        node.children().any(|node| find_usage(name, node))
    }
}

fn find_dead_code(node: SyntaxNode<NixLanguage>, results: &mut Vec<ResultItem>) {
    match node.kind() {
        SyntaxKind::NODE_LAMBDA => {
            let lambda = Lambda::cast(node.clone())
                .expect("Lambda::cast");
            if let Some(arg) = lambda.arg() {
                match arg.kind() {
                    SyntaxKind::NODE_IDENT => {
                        let name = Ident::cast(arg.clone())
                            .expect("Ident::cast");
                        if !find_usage(&name, node.clone()) {
                            results.push(ResultItem {
                                kind: ResultKind::LambdaArg,
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
                                results.push(ResultItem {
                                    kind: ResultKind::LambdaAt,
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
                                    results.push(ResultItem {
                                        kind: ResultKind::LambdaPattern,
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
                        results.push(ResultItem {
                            kind: ResultKind::LetInEntry,
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
                            results.push(ResultItem {
                                kind: ResultKind::LetInInherit,
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
        find_dead_code(child, results);
    }
}

fn main() {
    for path in args().skip(1) {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading file: {}", err);
                continue;
            }
        };

        let ast = rnix::parse(&content);
        let mut failed = false;
        for error in ast.errors() {
            eprintln!("Parse error: {}", error);
            failed = true;
        }
        if failed {
            continue;
        }

        let mut results = Vec::new();
        find_dead_code(ast.node(), &mut results);

        if results.len() == 0 {
            return;
        }

        let mut lines = Vec::new();
        let mut offset = 0;
        while let Some(next) = content[offset..].find('\n') {
            let line = &content[offset..offset + next];
            lines.push((offset, line));
            offset += next + 1;
        }
        lines.push((offset, &content[offset..]));

        let mut last_line = 0;
        let mut result_by_lines = Vec::new();
        for result in results {
            let range = result.node.text_range();
            let start = usize::from(range.start());
            let line_number = lines.iter().filter(|(offset, _)| *offset <= start).count();
            if line_number != last_line {
                last_line = line_number;
                result_by_lines.push((line_number, Vec::new()));
            }
            let result_by_lines_len = result_by_lines.len();
            let line_results = &mut result_by_lines[result_by_lines_len - 1].1;
            line_results.push(result);
        }
        for (line_number, results) in result_by_lines.iter_mut() {
            // file location
            println!("{}:{}:", path, line_number);
            // line
            println!("> {}", lines[*line_number - 1].1);
            results.sort_unstable_by_key(|result| result.node.text_range().start());

            // underscores ^^^^^^^^^
            let line_start = lines[*line_number - 1].0;
            let mut pos = line_start;
            print!("> ");
            for result in results.iter() {
                let range = result.node.text_range();
                let start = usize::from(range.start());
                let end = usize::from(range.end());
                print!("{0: <1$}{2:^<3$}", "", start - pos, "", end - start);
                pos = end;
            }
            println!("");

            let mut bars = String::new();
            let mut pos = line_start;
            for result in results.iter() {
                let range = result.node.text_range();
                let start = usize::from(range.start());
                bars = format!("{}{1: <2$}|", bars, "", start - pos);
                pos = start + 1;
            }
            println!("> {}", bars);

            // messages
            for i in (0..results.len()).rev() {
                let result = &results[i];
                let range = result.node.text_range();
                let start = usize::from(range.start());
                println!("> {}unused {}: {}", &bars[..start - line_start], result.kind, result.name.as_str());
            }
        }
    }
}
