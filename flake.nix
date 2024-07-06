{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { fenix, crane, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        let
          commonEnvironment = {
            SQLX_OFFLINE = "true";
          };

          toolchain = fenix.packages.${system}.complete;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain.toolchain;

          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
        in
        {
          packages.default =
            let
              pkgName = "rika";
              pkg = craneLib.buildPackage (
                {
                  src = ./.;
                  buildInputs = buildInputs;
                  nativeBuildInputs = nativeBuildInputs;
                }
                // commonEnvironment
              );
            in
            pkgs.dockerTools.buildLayeredImage {
              name = pkgName;
              tag = "latest";
              config.Cmd = "${pkg}/bin/${pkgName}";
              config.Expose = "3030";
            };

          devShells.default = pkgs.mkShell (
            {
              DATABASE_URL = "postgres://rika:rika@localhost:5432/rika";
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];

              packages =
                (with pkgs; [
                  nixfmt-rfc-style
                  sqlx-cli
                ])
                ++ (with toolchain; [
                  clippy
                  rustfmt
                  rust-analyzer
                ])
                ++ buildInputs
                ++ nativeBuildInputs;
            }
            // commonEnvironment
          );
        };
    };
}
