{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    let
      inherit (nixpkgs) lib;
      deadnixLambda = pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "deadnix";
          version = self.sourceInfo.lastModifiedDate;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postCheck = ''
            cargo clippy --all --all-features --tests -- \
                -D clippy::pedantic \
                -D warnings \
                -A clippy::module-name-repetitions \
                -A clippy::too-many-lines \
                -A clippy::cast-possible-wrap \
                -A clippy::cast-possible-truncation \
                -A clippy::nonminimal_bool \
                -A clippy::must-use-candidate \
                -A clippy::missing-panics-doc
          '';
          meta.description = "Scan Nix files for dead code";
        };
    in
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          packages = {
            default = self.packages."${system}".deadnix;
            deadnix = deadnixLambda pkgs;
          };

          apps.default = utils.lib.mkApp {
            drv = self.packages."${system}".default;
          };

          devShells.default = with pkgs; mkShell {
            nativeBuildInputs = [ cargo rustc rustfmt rustPackages.clippy rust-analyzer libiconv ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
        })
    // {
      overlays.default = (_: prev: {
        deadnix = deadnixLambda prev;
      });
    };
}
