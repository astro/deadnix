# deadnix

Scan `.nix` files for dead code (unused variable bindings).

## Usage with Nix Flakes

### Help

```
$ nix run github:astro/deadnix -- --help
```

```
USAGE:
    deadnix [FLAGS] [FILE_PATHS]...

FLAGS:
    -e, --edit             Remove unused code and write to source file
    -l, --no-lambda-arg    Don't check lambda parameter arguments
    -_, --no-underscore    Don't check any bindings that start with a _
    -q, --quiet            Don't print dead code report
    -h, --help             Prints help information
    -V, --version          Prints version information

ARGS:
    <FILE_PATHS>...    .nix files
```

### Scan for unused code

```
$ nix run github:astro/deadnix test.nix
```

```
test.nix:1:
> unusedArgs@{ unusedArg, usedArg, ... }:
> ^^^^^^^^^^   ^^^^^^^^^
> |            |
> |            Unused lambda pattern: unusedArg
> Unused lambda pattern: unusedArgs
test.nix:3:
>   inherit (builtins) unused_inherit;
>                      ^^^^^^^^^^^^^^
>                      |
>                      Unused let binding: unused_inherit
test.nix:5:
>   unused = "fnord";
>   ^^^^^^
>   |
>   Unused let binding: unused
test.nix:10:
>   shadowed = 42;
>   ^^^^^^^^
>   |
>   Unused let binding: shadowed
test.nix:11:
>   _unused = unused: false;
>   ^^^^^^^   ^^^^^^
>   |         |
>   |         Unused lambda argument: unused
>   Unused let binding: _unused
test.nix:13:
>   x = { unusedArg2, x ? args.y, ... }@args: used1 + x;
>         ^^^^^^^^^^
>         |
>         Unused lambda pattern: unusedArg2
```

### Remove unused code automatically

*Do commit* your changes into version control *before!*

```
$ nix run github:astro/deadnix -- -eq test.nix
```

## What if the produced reports are wrong?

Please open an issue. Do not forget to include the `.nix` code that
produces incorrect results.


## Commercial Support

The author can be hired to implement the features that you wish, or to
integrate this tool into your toolchain.
