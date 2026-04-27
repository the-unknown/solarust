{
  description = "Terminal animation of a randomly generated solar system";

  inputs = {
    nixpkgs.url = "git+https://github.com/NixOS/nixpkgs?shallow=1&ref=nixos-unstable"; #shallow clone, alot quicker build time
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          solarust = pkgs.rustPlatform.buildRustPackage {
            pname = "solarust";
            version = "0.1.1";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = with pkgs.lib; {
              description = "A terminal animation of a randomly generated solar system";
              homepage = "https://github.com/the-unknown/solarust";
              license = licenses.asl20;
              mainProgram = "solarust";
            };
          };

          default = self.packages.${system}.solarust;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
            ];
          };

          full = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              clippy
              rustfmt
            ];
          };
        };
      }
    );
}
