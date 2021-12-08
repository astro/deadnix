use std::fs;

mod scope;
mod binding;
mod usage;
mod dead_code;
mod dead_code_tests;
mod report;
mod edit;
mod edit_tests;


fn main() {
    let matches = clap::App::new("deadnix")
        .version(env!("CARGO_PKG_VERSION"))
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
        .arg(clap::Arg::with_name("QUIET")
             .short("q")
             .long("quiet")
             .help("Don't print dead code report")
        )
        .arg(clap::Arg::with_name("EDIT")
             .short("e")
             .long("edit")
             .help("Remove unused code and write to source file")
        )
        .arg(clap::Arg::with_name("FILE_PATHS")
             .multiple(true)
             .help(".nix files, or directories with .nix files inside")
        )
        .get_matches();

    let settings = dead_code::Settings {
        no_lambda_arg: matches.is_present("NO_LAMBDA_ARG"),
        no_underscore: matches.is_present("NO_UNDERSCORE"),
    };
    let quiet = matches.is_present("QUIET");
    let edit = matches.is_present("EDIT");

    let file_paths = matches.values_of("FILE_PATHS")
        .expect("FILE_PATHS");
    let files = file_paths.flat_map(|path| {
        let meta = fs::metadata(path)
            .expect("fs::metadata");
        let files: Box<dyn Iterator<Item = String>> =
            if meta.is_dir() {
                Box::new(
                    walkdir::WalkDir::new(path)
                        .into_iter()
                        .map(|result| result.unwrap().path().display().to_string())
                        .filter(|path| path.ends_with(".nix"))
                )
            } else {
                Box::new(
                    Some(path.to_string()).into_iter()
                )
            };
        files
    });
    for file in files {
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading file {}: {}", file, err);
                continue;
            }
        };

        let ast = rnix::parse(&content);
        let mut failed = false;
        for error in ast.errors() {
            eprintln!("Error parsing file {}: {}", file, error);
            failed = true;
        }
        if failed {
            continue;
        }

        let results = settings.find_dead_code(ast.node());
        if !quiet && results.len() > 0 {
            crate::report::Report::new(file.to_string(), &content, results.clone())
                .print();
        }
        if edit {
            let new_ast = crate::edit::edit_dead_code(
                &content,
                ast.node(),
                results.into_iter()
            );
            fs::write(file, new_ast)
                .expect("fs::write");
        }
    }
}
