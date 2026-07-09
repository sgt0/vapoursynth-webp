{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };
}
