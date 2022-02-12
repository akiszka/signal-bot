let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell {
  buildInputs = [
    pkgs.signal-cli
    rust.rust
    rust.rust-src
    # keep this line if you use bash
    pkgs.bashInteractive
  ];

  XDG_DATA_HOME = builtins.toString ./data;
  RUST_SRC_PATH = "${rust.rust-src}/lib/rustlib/src/rust/library";
}
