use std::{env::args, fs};

mod scope;
mod binding;
mod usage;
mod dead_code;
mod dead_code_tests;
mod report;


fn main() {
    for file_path in args().skip(1) {
        let content = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading file {}: {}", file_path, err);
                continue;
            }
        };

        let ast = rnix::parse(&content);
        let mut failed = false;
        for error in ast.errors() {
            eprintln!("Error parsing file {}: {}", file_path, error);
            failed = true;
        }
        if failed {
            continue;
        }

        let results = crate::dead_code::find_dead_code(ast.node());
        if results.len() > 0 {
            crate::report::Report::new(file_path, &content, results)
                .print();
        }
    }
}
