{ system ? builtins.currentSystem }:

let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  signalbot = import ./package.nix { inherit sources pkgs; };

  name = "akiszka/signalbot";
  tag = "latest";

in pkgs.dockerTools.buildLayeredImage {
  inherit name tag;
  created = "now";
  contents = [ signalbot pkgs.signal-cli ./additional ./data ];

  config = {
    Cmd = [ "/bin/bot" ];
    Env = [ "ROCKET_ENV=release" "XDG_DATA_HOME=/" ];
    WorkingDir = "/";
  };
}
