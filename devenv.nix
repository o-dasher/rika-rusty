{ pkgs, ... }:
let
  name = "osaka";
in
{
  packages = with pkgs; [
    openssl
    sqlx-cli
    nixfmt-rfc-style
  ];

  dotenv.enable = true;
  env.DATABASE_URL = "postgres:///${name}";

  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  services.postgres = {
    enable = true;
    initialDatabases = [ { name = name; } ];
  };
}
