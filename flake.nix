{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    utils,
  }: let
    out = system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          pkg-config
          openssl
        ];
      };
    };
  in
    with utils.lib; eachSystem defaultSystems out;
}

# vim: et ts=2 sw=2 sts=2 ai si ft=nix
