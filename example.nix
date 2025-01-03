unusedArgs@{ unusedArg, usedArg, ... }:
let
  inherit (builtins) unused_inherit;
  inherit (used2) used_inherit;
  unused = "fnord";
  used1 = "important";
  used2 = usedArg;
  used3 = used4: "k.${used4}";
  used4 = { t = used_inherit; };
  shadowed = 42;
  _unused = unused: false;
  _used = 23;
in {
  x = { unusedArg2, x ? args.y, ... }@args: used1 + x;
  inherit used2;
  "${used3}" = true;
  y = used4.t;
  z = let shadowed = 23; in shadowed;
  inherit _used;
}
