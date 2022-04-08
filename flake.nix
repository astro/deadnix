{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      rust = fenix.packages.${system}.stable.withComponents [
        "cargo"
        "rustc"
        "rust-src"  # just for rust-analyzer
        "clippy"
      ];

      # Override the version used in naersk
      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };
    in rec {
      # `nix build`
      packages.deadnix = naersk-lib.buildPackage {
        pname = "deadnix";
        src = ./.;
        doCheck = true;
        cargoTestCommands = x:
          x ++ [
            # clippy
            ''cargo clippy --all --all-features --tests -- \
              -D clippy::pedantic \
              -D warnings \
              -A clippy::module-name-repetitions \
              -A clippy::too-many-lines \
              -A clippy::cast-possible-wrap \
              -A clippy::cast-possible-truncation \
              -A clippy::nonminimal_bool''
          ];
        meta.description = "Scan Nix files for dead code";
      };
      defaultPackage = packages.deadnix;

      checks = packages;

      hydraJobs =
        let
          hydraSystems = [
            "x86_64-linux"
            "aarch64-linux"
          ];
        in
          if builtins.elem system hydraSystems
          then builtins.mapAttrs (_: nixpkgs.lib.hydraJob) checks
          else {};

      # `nix run`
      apps.deadnix = utils.lib.mkApp {
        drv = packages.deadnix;
      };
      defaultApp = apps.deadnix;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = [
          fenix.packages.${system}.rust-analyzer
        ] ++
        (with defaultPackage; nativeBuildInputs ++ buildInputs);
      };
    }) // {
      overlay = final: prev: {
        inherit (self.packages.${prev.system})
          deadnix
        ;
      };
    };
}
