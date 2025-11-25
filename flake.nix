{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      forAllSystems = function: lib.genAttrs
        [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ]
        (system: function nixpkgs.legacyPackages.${system});

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
    in {
      packages = forAllSystems (pkgs: {
        default = self.packages.${pkgs.system}.deadnix;
        deadnix = deadnixLambda pkgs;
      });

      checks = self.packages;

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = lib.getExe self.packages.${pkgs.system}.default;
        };
      });

      devShells = forAllSystems (pkgs: {
        default = with pkgs; mkShell {
          nativeBuildInputs = [ cargo rustc rustfmt rustPackages.clippy rust-analyzer libiconv ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });

      overlays.default = (final: _: {
        deadnix = deadnixLambda final;
      });
    };
}
