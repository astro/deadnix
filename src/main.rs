use std::fs;

mod scope;
mod binding;
mod usage;
mod dead_code;
mod dead_code_tests;
mod report;


fn main() {
    let matches = clap::App::new("deadnix")
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(clap::Arg::with_name("NO_LAMBDA_ARG")
             .short("l")
             .long("no-lambda-arg")
             .help("Don't check lambda parameter arguments")
        )
        .arg(clap::Arg::with_name("NO_UNDERSCORE")
             .short("_")
             .long("no-underscore")
             .help("Don't check any bindings that start with a _")
        )
        .arg(clap::Arg::with_name("FILE_PATHS")
             .multiple(true)
             .help(".nix files")
        )
        .get_matches();

    let settings = dead_code::Settings {
        no_lambda_arg: matches.is_present("NO_LAMBDA_ARG"),
        no_underscore: matches.is_present("NO_UNDERSCORE"),
    };

    let file_paths = matches.values_of("FILE_PATHS")
        .expect("FILE_PATHS");
    for file_path in file_paths {
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

        let results = settings.find_dead_code(ast.node());
        if results.len() > 0 {
            crate::report::Report::new(file_path.to_string(), &content, results)
                .print();
        }
    }
}
