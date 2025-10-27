{
  description = "Set of I/O-free Rust coroutines to manage calendars";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/staging-next";
    };
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pimalaya = {
      url = "github:pimalaya/nix";
      flake = false;
    };
  };

  outputs =
    inputs:
    (import inputs.pimalaya).mkFlakeOutputs inputs {
      shell = ./shell.nix;
    };
}
