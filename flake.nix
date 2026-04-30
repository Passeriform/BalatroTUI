{
  description = "Flake for BalatroTUI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:

    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustPlatform = pkgs.rustPackages;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "balatro_tui";
          version = "0.1.4";

          src = pkgs.fetchFromGitHub {
            owner = "Passeriform";
            repo = "BalatroTUI";
            tag = "v${version}";
            hash = "sha256-BcmGZXipzlr2GeYWwGEukLWqKxOdF0jOA3lZQnGtPDY=";
          };

          cargoHash = "sha256-Jlf3PvphAYDUoDsW82gufStLgE0ocHlBPMGEB73CSY0=";

          meta = {
            description = "A TUI clone of Balatro, a game by LocalThunk";
            homepage = "https://github.com/Passeriform/BalatroTUI";
            license = pkgs.lib.licenses.gpl3Only;
            maintainers = with pkgs.lib.maintainers; [ ];
            mainProgram = "balatro_tui";
          };
        };
      }
    );
}
