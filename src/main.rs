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
    let matches = clap::Command::new("deadnix")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(
            clap::Arg::new("NO_LAMBDA_ARG")
                .short('l')
                .long("no-lambda-arg")
                .help("Don't check lambda parameter arguments"),
        )
        .arg(
            clap::Arg::new("NO_LAMBDA_PATTERN_NAMES")
                .short('L')
                .long("no-lambda-pattern-names")
                .help("Don't check lambda attrset pattern names (don't break nixpkgs callPackage)"),
        )
        .arg(
            clap::Arg::new("NO_UNDERSCORE")
                .short('_')
                .long("no-underscore")
                .help("Don't check any bindings that start with a _"),
        )
        .arg(
            clap::Arg::new("QUIET")
                .short('q')
                .long("quiet")
                .help("Don't print dead code report"),
        )
        .arg(
            clap::Arg::new("EDIT")
                .short('e')
                .long("edit")
                .help("Remove unused code and write to source file"),
        )
        .arg(
            clap::Arg::new("HIDDEN")
                .short('h')
                .long("hidden")
                .help("Recurse into hidden subdirectories and process hidden .*.nix files"),
        )
        .arg(
            clap::Arg::new("FAIL_ON_REPORTS")
                .short('f')
                .long("fail")
                .help("Exit with 1 if unused code has been found"),
        )
        .arg(
            clap::Arg::new("OUTPUT_FORMAT")
                .short('o')
                .long("output-format")
                .takes_value(true)
                .possible_value("human-readable")
                .possible_value("json")
                .default_value("human-readable")
                .help("Output format to use"),
        )
        .arg(
            clap::Arg::new("FILE_PATHS")
                .multiple_values(true)
                .default_value(".")
                .help(".nix files, or directories with .nix files inside"),
        )
        .get_matches();

    let fail_on_reports = matches.is_present("FAIL_ON_REPORTS");
    let mut report_count = 0;

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
    let output_format = matches.value_of("OUTPUT_FORMAT");

    let file_paths = matches.values_of("FILE_PATHS").expect("FILE_PATHS");
    let files = file_paths.flat_map(|path| {
        let meta = fs::metadata(path).expect("fs::metadata");
        let files: Box<dyn Iterator<Item = String>> = if meta.is_dir() {
            Box::new(
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_entry(is_visible)
                    .map(Result::unwrap)
                    .filter(|entry| {
                        entry.file_type().is_file()
                            && entry.path().extension().map_or(false, |ext| ext.eq_ignore_ascii_case("nix"))
                    })
                    .map(|entry| entry.path().display().to_string())
                ,
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
        report_count += results.len();
        if !quiet && !results.is_empty() {
            match output_format {
                Some("human-readable") => crate::report::print(file.to_string(), &content, &results),
                #[cfg(feature = "json-out")]
                Some("json") => crate::report::print_json(&file.to_string(), &content, &results),
                #[cfg(not(feature = "json-out"))]
                Some("json") => println!("`deadnix` needs to be built with `json-out` feature flag for JSON output format."),
                _ => println!("Unknown output format."), // clap shouldn't allow this case
            };
        }
        if edit {
            let (new_ast, has_changes) = crate::edit::edit_dead_code(&content, results.into_iter());
            if has_changes {
                fs::write(file, new_ast).expect("fs::write");
            }
        }
    }

    if fail_on_reports && report_count > 0 {
        std::process::exit(1);
    }
}
