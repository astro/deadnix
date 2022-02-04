# deadnix

Scan `.nix` files for dead code (unused variable bindings).

## Usage with Github Actions

See [deadnix-action](https://github.com/astro/deadnix-action)


## Usage with Nix Flakes

### Help

```
$ nix run github:astro/deadnix -- --help
```

```
USAGE:
    deadnix [OPTIONS] [FILE_PATHS]...

ARGS:
    <FILE_PATHS>...    .nix files, or directories with .nix files inside

OPTIONS:
    -_, --no-underscore              Don't check any bindings that start with a _
    -e, --edit                       Remove unused code and write to source file
    -f, --fail                       Exit with 1 if unused code has been found
    -h, --hidden                     Recurse into hidden subdirectories and process hidden .*.nix
                                     files
        --help                       Print help information
    -l, --no-lambda-arg              Don't check lambda parameter arguments
    -L, --no-lambda-pattern-names    Don't check lambda attrset pattern names (don't break nixpkgs
                                     callPackage)
    -q, --quiet                      Don't print dead code report
    -V, --version                    Print version information
```

### Scan for unused code

```
$ nix run github:astro/deadnix test.nix
```

```
Warning: Unused declarations were found.
    ╭─[example.nix:1:1]
    │
  1 │ unusedArgs@{ unusedArg, usedArg, ... }:
    · ─────┬────   ────┬────
    ·      │           ╰────── Unused lambda pattern: unusedArg
    ·      │
    ·      ╰────────────────── Unused lambda pattern: unusedArgs
  3 │   inherit (builtins) unused_inherit;
    ·                      ───────┬──────
    ·                             ╰──────── Unused let binding: unused_inherit
  5 │   unused = "fnord";
    ·   ───┬──
    ·      ╰──── Unused let binding: unused
 10 │   shadowed = 42;
    ·   ────┬───
    ·       ╰───── Unused let binding: shadowed
 11 │   _unused = unused: false;
    ·   ───┬───   ───┬──
    ·      │         ╰──── Unused lambda argument: unused
    ·      │
    ·      ╰────────────── Unused let binding: _unused
 13 │   x = { unusedArg2, x ? args.y, ... }@args: used1 + x;
    ·         ─────┬────
    ·              ╰────── Unused lambda pattern: unusedArg2
────╯
```


### Remove unused code automatically

**Do commit** your changes into version control **before!**

```
$ nix run github:astro/deadnix -- -eq test.nix
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
