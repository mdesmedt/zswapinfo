{
  description = "zswapinfo - show the current state of zswap";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAll (pkgs: rec {
        zswapinfo = pkgs.rustPlatform.buildRustPackage {
          pname = "zswapinfo";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
        default = zswapinfo;
      });

      devShells = forAll (
        pkgs: with pkgs; {
          default = mkShell {
            packages = [
              cargo
              rustc
              rust-analyzer
              clippy
              rustfmt
            ];
          };
        }
      );
    };
}
