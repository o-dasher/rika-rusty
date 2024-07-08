{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-filter.url = "github:numtide/nix-filter";

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
    {
      fenix,
      crane,
      nix-filter,
      ...
    }@inputs:
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

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        in
        {
          packages =
            let
              pkgName = "osaka";
              pkg = craneLib.buildPackage (
                {
                  buildInputs = buildInputs;
                  nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];

                  src = nix-filter.lib {
                    root = ./.;
                    include = [
                      "Cargo.toml"
                      "Cargo.lock"
                      "src"
                      "migrations"
                      ".sqlx"
                    ];
                  };

                  postInstall = ''
                    wrapProgram $out/bin/${pkgName} \
                        --prefix LD_LIBRARY_PATH : "${LD_LIBRARY_PATH}"
                  '';
                }
                // commonEnvironment
              );
            in
            {
              default = pkg;
              docker = pkgs.dockerTools.buildLayeredImage {
                tag = "latest";
                name = pkgName;
                contents = with pkgs; [ cacert ];
                config.Cmd = [ "${pkg}/bin/${pkgName}" ];
              };
            };

          devShells.default = pkgs.mkShell (
            {
              inherit LD_LIBRARY_PATH;

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
