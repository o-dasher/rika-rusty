{ pkgs, ... }:
{
  packages = with pkgs; [
    openssl
  ];

  dotenv.enable = true;
  languages.rust = {
    enable = true;
    channel = "nightly";
  };
}
