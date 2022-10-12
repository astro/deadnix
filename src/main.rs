use std::fs;
use clap::{Arg, ArgAction, Command};

mod binding;
mod dead_code;
mod dead_code_tests;
mod edit;
mod edit_tests;
mod report;
mod scope;
mod usage;

#[derive(Clone, Copy, Debug)]
enum OutputFormat {
    HumanReadable,
    #[cfg(feature = "json-out")]
    Json,
}

fn main() {
    let matches = Command::new("deadnix")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Astro <astro@spaceboyz.net>")
        .about("Find dead code in .nix files")
        .arg(
            Arg::new("NO_LAMBDA_ARG")
                .action(ArgAction::SetTrue)
                .short('l')
                .long("no-lambda-arg")
                .help("Don't check lambda parameter arguments"),
        )
        .arg(
            Arg::new("NO_LAMBDA_PATTERN_NAMES")
                .action(ArgAction::SetTrue)
                .short('L')
                .long("no-lambda-pattern-names")
                .help("Don't check lambda attrset pattern names (don't break nixpkgs callPackage)"),
        )
        .arg(
            Arg::new("NO_UNDERSCORE")
                .action(ArgAction::SetTrue)
                .short('_')
                .long("no-underscore")
                .help("Don't check any bindings that start with a _"),
        )
        .arg(
            Arg::new("QUIET")
                .action(ArgAction::SetTrue)
                .short('q')
                .long("quiet")
                .help("Don't print dead code report"),
        )
        .arg(
            Arg::new("EDIT")
                .action(ArgAction::SetTrue)
                .short('e')
                .long("edit")
                .help("Remove unused code and write to source file"),
        )
        .arg(
            Arg::new("HIDDEN")
                .action(ArgAction::SetTrue)
                .short('h')
                .long("hidden")
                .help("Recurse into hidden subdirectories and process hidden .*.nix files"),
        )
        .arg(
            Arg::new("FAIL_ON_REPORTS")
                .action(ArgAction::SetTrue)
                .short('f')
                .long("fail")
                .help("Exit with 1 if unused code has been found"),
        )
        .arg(
            Arg::new("OUTPUT_FORMAT")
                .short('o')
                .long("output-format")
                .value_parser([
                    "human-readable",
                    #[cfg(feature = "json-out")]
                    "json",
                ])
                .default_value("human-readable")
                .help("Output format to use"),
        )
        .arg(
            Arg::new("FILE_PATHS")
                .num_args(1..)
                .default_value(".")
                .help(".nix files, or directories with .nix files inside"),
        )
        .get_matches();

    let fail_on_reports = matches.get_flag("FAIL_ON_REPORTS");
    let mut report_count = 0;

    let settings = dead_code::Settings {
        no_lambda_arg: matches.get_flag("NO_LAMBDA_ARG"),
        no_lambda_pattern_names: matches.get_flag("NO_LAMBDA_PATTERN_NAMES"),
        no_underscore: matches.get_flag("NO_UNDERSCORE"),
    };
    let quiet = matches.get_flag("QUIET");
    let edit = matches.get_flag("EDIT");
    let is_visible = if matches.get_flag("HIDDEN") {
        |_: &walkdir::DirEntry| true
    } else {
        |entry: &walkdir::DirEntry| entry.file_name()
            .to_str()
            .map_or(false, |s| s == "." || ! s.starts_with('.'))
    };
    let output_format = match matches.get_one::<&str>("OUTPUT_FORMAT") {
        Some(&"human-readable") => OutputFormat::HumanReadable,
        #[cfg(feature = "json-out")]
        Some(&"json") => OutputFormat::Json,
        #[cfg(not(feature = "json-out"))]
        Some(&"json") => println!("`deadnix` needs to be built with `json-out` feature flag for JSON output format."),
        _ => unreachable!(), // clap shouldn't allow this case
    };

    let file_paths = matches.get_many::<String>("FILE_PATHS").expect("FILE_PATHS");
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
                OutputFormat::HumanReadable =>
                    crate::report::print(file.to_string(), &content, &results),

                #[cfg(feature = "json-out")]
                OutputFormat::Json =>
                    crate::report::print_json(&file.to_string(), &content, &results),
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
