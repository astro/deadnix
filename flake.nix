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
          src = self;
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
          meta = with lib; {
            description = "Find and remove unused code in .nix source files";
            homepage = "https://github.com/astro/deadnix";
            license = licenses.gpl3Only;
            mainProgram = "deadnix";
            maintainers = with maintainers; [ astro ];
          };
        };
    in
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          packages = {
            default = self.packages."${system}".deadnix;
            deadnix = deadnixLambda pkgs;
          };
        in
        {
          inherit packages;

          checks = packages;

          apps.default = utils.lib.mkApp {
            drv = self.packages."${system}".default;
          };

          devShells.default = with pkgs; mkShell {
            nativeBuildInputs = [ cargo rustc rustfmt rustPackages.clippy rust-analyzer libiconv ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
        })
    // {
      overlays.default = (final: _: {
        deadnix = deadnixLambda final;
      });
    };
}
