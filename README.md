# deadnix

Scan `.nix` files for dead code (unused variable bindings).

## Usage with Github Actions

See [deadnix-action](https://github.com/astro/deadnix-action)


## Usage with Nix Flakes

### Help

```console
$ nix run github:astro/deadnix -- --help
Find dead code in .nix files

Usage: deadnix [OPTIONS] [FILE_PATHS]...

Arguments:
  [FILE_PATHS]...  .nix files, or directories with .nix files inside [default: .]

Options:
  -l, --no-lambda-arg                  Don't check lambda parameter arguments
  -L, --no-lambda-pattern-names        Don't check lambda attrset pattern names (don't break nixpkgs callPackage)
  -_, --no-underscore                  Don't check any bindings that start with a _
  -q, --quiet                          Don't print dead code report
  -e, --edit                           Remove unused code and write to source file
  -h, --hidden                         Recurse into hidden subdirectories and process hidden .*.nix files
      --help
  -f, --fail                           Exit with 1 if unused code has been found
  -o, --output-format <OUTPUT_FORMAT>  Output format to use [default: human-readable] [possible values: human-readable, json]
      --exclude <EXCLUDES>...          Files to exclude from analysis
  -V, --version                        Print version
```

Reports contain ANSI color escape codes unless the
[`$NO_COLOR`](https://no-color.org/) environment variable is set.

The `--exclude` parameter accepts multiple paths. Separate them with
`--` to pass `[FILE_PATHS]...`.

### Scan for unused code

```console
$ nix run github:astro/deadnix example.nix
Warning: Unused declarations were found.
    ╭─[example.nix:1:1]
  1 │unusedArgs@{ unusedArg, usedArg, ... }:
    ·     │           ╰───── Unused lambda pattern: unusedArg
    ·     ╰───────────────── Unused lambda pattern: unusedArgs
  3 │  inherit (builtins) unused_inherit;
    ·                            ╰─────── Unused let binding: unused_inherit
  5 │  unused = "fnord";
    ·     ╰─── Unused let binding: unused
 10 │  shadowed = 42;
    ·      ╰──── Unused let binding: shadowed
 11 │  _unused = unused: false;
    ·     │         ╰─── Unused lambda argument: unused
    ·     ╰───────────── Unused let binding: _unused
 13 │  x = { unusedArg2, x ? args.y, ... }@args: used1 + x;
    ·             ╰───── Unused lambda pattern: unusedArg2
```


### Remove unused code automatically

**Do commit** your changes into version control **before!**

```console
$ nix run github:astro/deadnix -- -eq test.nix
```

## Usage with [pre-commit](https://pre-commit.com/)

Add the following to your project's `.pre-commit-config.yaml`:
```yaml
repos:
  - repo: https://github.com/astro/deadnix
    rev: ID # frozen: VERSION
    hooks:
      - id: deadnix
        #args: [--edit] # Uncomment to automatically modify files
        stages: [commit]
```

Replace `ID` and `VERSION` above with the relevant version tag and
commit ID for reference, for example:

```yaml
rev: da39a3ee5e6b4b0d3255bfef95601890afd80709  # frozen: v1.2.3
```

## Behavior

### Renaming of all unused to lambda args to start with `_`

If you disfavor marking them as unused, use option `--no-lambda-arg`.


### nixpkgs `callPackages` with multiple imports

`callPackages` guesses the packages to inject by the names of a
packages' lambda attrset pattern names. Some packages alias these with
`@args` to pass them to another `import ...nix args`.

As the used args are only named in the imported file they will be
recognized as dead in the package source file that is imported by
`callPackage`, rendering it unable to guess the dependencies to call
the packages with.

Use option `--no-lambda-pattern-names` in this case.


## What if the produced reports are wrong?

Please open an issue. Do not forget to include the `.nix` code that
produces incorrect results.


## Commercial Support

The author can be hired to implement the features that you wish, or to
integrate this tool into your toolchain.
