let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell {
  buildInputs = [
    pkgs.signal-cli

    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}
