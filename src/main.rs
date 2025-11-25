use clap::{Arg, ArgAction, Command};
#[cfg(feature = "json-out")]
use serde_json::json;
use std::{collections::HashSet, fs, path::Path};

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
                .help(
                    "Don't check any bindings that start with a _\n\
                       (Lambda arguments starting with _ are not checked anyway.)",
                ),
        )
        .arg(
            Arg::new("WARN_USED_UNDERSCORE")
                .action(ArgAction::SetTrue)
                .short('W')
                .long("warn-used-underscore")
                .help("Warn if bindings are referenced that start with '_'"),
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
        // taken by "HIDDEN"
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .long("help")
                .global(true)
                .action(ArgAction::Help),
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
                .value_parser(["human-readable", "json"])
                .default_value("human-readable")
                .help("Output format to use"),
        )
        .arg(
            Arg::new("EXCLUDES")
                .long("exclude")
                .num_args(1..)
                .help("Files to exclude from analysis"),
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
        warn_used_underscore: matches.get_flag("WARN_USED_UNDERSCORE"),
    };
    let quiet = matches.get_flag("QUIET");
    let edit = matches.get_flag("EDIT");
    let is_visible = if matches.get_flag("HIDDEN") {
        |_: &walkdir::DirEntry| true
    } else {
        |entry: &walkdir::DirEntry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|s| s == "." || s == ".." || !s.starts_with('.'))
        }
    };
    let is_included: Box<dyn Fn(&String) -> bool> =
        if let Some(excludes) = matches.get_many::<String>("EXCLUDES") {
            let excludes = excludes
                .filter_map(|exclude| Path::new(exclude).canonicalize().ok())
                .collect::<HashSet<_>>();
            Box::new(move |s| {
                let s = Path::new(s).canonicalize().unwrap();
                !excludes.iter().any(|exclude| s.starts_with(exclude))
            })
        } else {
            Box::new(|_| true)
        };
    let output_format = matches
        .get_one::<String>("OUTPUT_FORMAT")
        .map(String::as_str);
    let output_format = match output_format {
        Some("human-readable") => OutputFormat::HumanReadable,
        #[cfg(feature = "json-out")]
        Some("json") => OutputFormat::Json,
        #[cfg(not(feature = "json-out"))]
        Some("json") => panic!(
            "`deadnix` needs to be built with `json-out` feature flag for JSON output format."
        ),
        _ => panic!("Unknown output format."), // clap shouldn't allow this case
    };

    let file_paths = matches
        .get_many::<String>("FILE_PATHS")
        .expect("FILE_PATHS");
    let files = file_paths.flat_map(|path| {
        let meta = fs::metadata(path);
        let files: Box<dyn Iterator<Item = String>> = match meta {
            // scan directory
            Ok(meta) if meta.is_dir() => Box::new(
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_entry(is_visible)
                    .map(Result::unwrap)
                    .filter(|entry| {
                        entry.file_type().is_file()
                            && entry
                                .path()
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("nix"))
                    })
                    .map(|entry| entry.path().display().to_string())
                    .filter(&is_included),
            ),

            // single file
            Ok(_) => Box::new([path.clone()].into_iter()),

            // error
            Err(error) => {
                match output_format {
                    OutputFormat::HumanReadable => {
                        eprintln!("Error stating file {path}: {error}");
                    }

                    #[cfg(feature = "json-out")]
                    OutputFormat::Json => println!(
                        "{}",
                        json!({
                            "file": path,
                            "results": [{
                                "message": format!("{error}"),
                            }],
                        })
                    ),
                }
                Box::new([].into_iter())
            }
        };
        files
    });
    for file in files {
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(error) => {
                match output_format {
                    OutputFormat::HumanReadable => {
                        eprintln!("Error reading file {file}: {error}");
                    }

                    #[cfg(feature = "json-out")]
                    OutputFormat::Json => println!(
                        "{}",
                        json!({
                            "file": file,
                            "results": [{
                                "message": format!("{error}"),
                            }],
                        })
                    ),
                }
                continue;
            }
        };

        let ast = rnix::Root::parse(&content);
        let errors = ast.errors();

        if !errors.is_empty() {
            match output_format {
                OutputFormat::HumanReadable => {
                    for error in errors {
                        eprintln!("Error parsing file {file}: {error}");
                    }
                }

                #[cfg(feature = "json-out")]
                OutputFormat::Json => println!(
                    "{}",
                    json!({
                        "file": file,
                        "results": errors.iter()
                            .map(|error| json!({
                                "message": format!("{error}"),
                            }))
                            .collect::<Vec<_>>(),
                    })
                ),
            }
            continue;
        }

        let results = settings.find_dead_code(&ast.syntax());
        report_count += results.len();
        if !quiet && !results.is_empty() {
            match output_format {
                OutputFormat::HumanReadable => {
                    crate::report::print(file.clone(), &content, &results);
                }

                #[cfg(feature = "json-out")]
                OutputFormat::Json => {
                    crate::report::print_json(&file.clone(), &content, &results);
                }
            }
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
