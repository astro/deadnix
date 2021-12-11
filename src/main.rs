use std::fs;

mod binding;
mod dead_code;
mod dead_code_tests;
mod edit;
mod edit_tests;
mod report;
mod scope;
mod usage;

fn main() {
    let matches = clap::App::new("deadnix")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(
            clap::Arg::with_name("NO_LAMBDA_ARG")
                .short("l")
                .long("no-lambda-arg")
                .help("Don't check lambda parameter arguments"),
        )
        .arg(
            clap::Arg::with_name("NO_LAMBDA_PATTERN_NAMES")
                .short("L")
                .long("no-lambda-pattern-names")
                .help("Don't check lambda attrset pattern names (don't break nixpkgs callPackage)"),
        )
        .arg(
            clap::Arg::with_name("NO_UNDERSCORE")
                .short("_")
                .long("no-underscore")
                .help("Don't check any bindings that start with a _"),
        )
        .arg(
            clap::Arg::with_name("QUIET")
                .short("q")
                .long("quiet")
                .help("Don't print dead code report"),
        )
        .arg(
            clap::Arg::with_name("EDIT")
                .short("e")
                .long("edit")
                .help("Remove unused code and write to source file"),
        )
        .arg(
            clap::Arg::with_name("HIDDEN")
                .short("h")
                .long("hidden")
                .help("Recurse into hidden subdirectories and process hidden .*.nix files"),
        )
        .arg(
            clap::Arg::with_name("FILE_PATHS")
                .multiple(true)
                .help(".nix files, or directories with .nix files inside"),
        )
        .get_matches();

    let settings = dead_code::Settings {
        no_lambda_arg: matches.is_present("NO_LAMBDA_ARG"),
        no_lambda_pattern_names: matches.is_present("NO_LAMBDA_PATTERN_NAMES"),
        no_underscore: matches.is_present("NO_UNDERSCORE"),
    };
    let quiet = matches.is_present("QUIET");
    let edit = matches.is_present("EDIT");
    let is_visible = if matches.is_present("HIDDEN") {
        |_: &walkdir::DirEntry| true
    } else {
        |entry: &walkdir::DirEntry| entry.file_name()
            .to_str()
            .map_or(false, |s| s == "." || ! s.starts_with('.'))
    };

    let file_paths = matches.values_of("FILE_PATHS").expect("FILE_PATHS");
    let files = file_paths.flat_map(|path| {
        let meta = fs::metadata(path).expect("fs::metadata");
        let files: Box<dyn Iterator<Item = String>> = if meta.is_dir() {
            Box::new(
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_entry(is_visible)
                    .into_iter()
                    .map(|result| result.unwrap().path().display().to_string())
                    .filter(|path| {
                        path.rsplit('.')
                            .next()
                            .map(|ext| ext.eq_ignore_ascii_case("nix"))
                            == Some(true)
                    }),
            )
        } else {
            Box::new(Some(path.to_string()).into_iter())
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

        let results = settings.find_dead_code(&ast.node());
        if !quiet && !results.is_empty() {
            crate::report::Report::new(file.to_string(), &content, results.clone()).print();
        }
        if edit {
            let new_ast = crate::edit::edit_dead_code(&content, results.into_iter());
            fs::write(file, new_ast).expect("fs::write");
        }
    }
}
