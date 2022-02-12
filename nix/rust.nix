# nix/rust.nix
{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  chan = pkgs.rustChannelOf { channel = "nightly"; date = "2022-02-04"; };
in chan