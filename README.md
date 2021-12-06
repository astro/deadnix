# deadnix

Scan `.nix` files for dead code (unused variable bindings).

## Usage with Nix Flakes

```
$ nix run github:astro/deadnix test.nix
```

```
test.nix:1:
> unusedArgs@{ unusedArg, usedArg, ... }:
> ^^^^^^^^^^   ^^^^^^^^^
> |            |
> |            unused lambda pattern: unusedArg
> unused lambda @-binding: unusedArgs
test.nix:5:
>   unused = "fnord";
>   ^^^^^^
>   |
>   unused let in binding: unused
test.nix:3:
>   inherit (builtins) unused_inherit;
>                      ^^^^^^^^^^^^^^
>                      |
>                      unused let in inherit binding: unused_inherit
test.nix:12:
>   x = { unusedArg2, x ? args.y, ... }@args: used1 + x;
>         ^^^^^^^^^^
>         |
>         unused lambda pattern: unusedArg2
```


## What if the produced reports are wrong?

Please open an issue. Do not forget to include the `.nix` code that
produces incorrect results.


## Commercial Support

The author can be hired to implement the features that you wish, or to
integrate this tool into your toolchain.
