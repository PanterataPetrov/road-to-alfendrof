{
  description = "Road to Alfendrof development tools";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/0bb2eedc20e4739f29e8045034f2828ad7054f8f";

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo rustc rust-analyzer rustfmt clippy pkg-config
              curl jq just nixd zsh (python3.withPackages (ps: [ ps.pillow ]))
            ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (with pkgs; [
              alsa-lib udev vulkan-loader libxkbcommon wayland
              xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
            ]);
            shellHook = ''
              export SHELL=${pkgs.zsh}/bin/zsh
            '';
          };
        });
    };
}
