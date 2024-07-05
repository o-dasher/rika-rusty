{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { devenv, self, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        {
          packages.devenv-up = self.devShells.${system}."default".config.procfileScript;
          devShells.default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules =
              let
                databaseName = "osaka";
              in
              [
                {
                  packages = with pkgs; [
                    openssl
                    sqlx-cli
                    nixfmt-rfc-style
                  ];

                  env.DATABASE_URL = "postgres:///${databaseName}";

                  languages.rust = {
                    enable = true;
                    channel = "nightly";
                  };

                  services.postgres = {
                    enable = true;
                    initialDatabases = [ { name = databaseName; } ];
                  };
                }
              ];
          };
        };
    };
}
