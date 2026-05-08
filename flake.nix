{
  description = "JayJay GPUI shell — Linux/Windows native frontend for jj.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    nix-appimage = {
      url = "github:ralismark/nix-appimage";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Linux only — the SwiftUI shell handles macOS, and bundling for Windows
  # is a separate effort (no AppImage there).
  outputs = { self, nixpkgs, flake-utils, crane, nix-appimage }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        # Runtime libs GPUI dlopens (Vulkan, Wayland, X11) and crates link to.
        runtimeDeps = with pkgs; [
          fontconfig
          wayland
          libxkbcommon
          xorg.libxcb
          xorg.libX11
          vulkan-loader
          openssl
          zstd
        ];

        # Keep cargo sources + shell/gpui/assets (include_bytes! at compile time).
        src = pkgs.lib.cleanSourceWith {
          src = pkgs.lib.cleanSource ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || (pkgs.lib.hasInfix "/assets/" path);
          name = "source";
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          cargoExtraArgs = "-p jayjay-gpui";

          nativeBuildInputs = with pkgs; [ pkg-config clang ];
          buildInputs = runtimeDeps;

          # Tests run in gpui-ci.yml; nix sandbox has no `jj` on PATH.
          doCheck = false;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        jayjay-gpui = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "jayjay-gpui";

          # rpath so the AppImage finds the dlopen'd libs after bundling.
          postFixup = ''
            patchelf --set-rpath "${pkgs.lib.makeLibraryPath runtimeDeps}" \
              $out/bin/jayjay-gpui || true
          '';
        });
      in
      {
        packages = {
          inherit jayjay-gpui;
          default = jayjay-gpui;
          appimage = nix-appimage.bundlers.${system}.default jayjay-gpui;
        };

        apps.default = flake-utils.lib.mkApp { drv = jayjay-gpui; };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ jayjay-gpui ];
          packages = with pkgs; [ rustc cargo clippy rustfmt ];
        };
      }
    );
}
