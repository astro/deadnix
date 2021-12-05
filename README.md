# deadnix

Scan `.nix` files for dead code (unused variable bindings).

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
