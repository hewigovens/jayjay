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
          libxcb
          libx11
          libdrm
          libxshmfence
          expat
          zlib
          libffi
          libbsd
          libmd
          stdenv.cc.cc.lib
          vulkan-loader
          openssl
          zstd
          xorg.libXau
          xorg.libXdmcp
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
          pname = "jayjay-gpui";
          version = "0.1.0-alpha";
          cargoExtraArgs = "-p jayjay-gpui";

          nativeBuildInputs = with pkgs; [ pkg-config clang makeWrapper librsvg ];
          buildInputs = runtimeDeps;

          # Tests run in gpui-ci.yml; nix sandbox has no `jj` on PATH.
          doCheck = false;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        jayjay-gpui = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          postInstall = ''
            install -Dm644 ${./shell/gpui/linux/dev.hewig.JayJay.desktop} \
              $out/share/applications/dev.hewig.JayJay.desktop
            install -Dm644 ${./shell/gpui/linux/dev.hewig.JayJay.metainfo.xml} \
              $out/share/metainfo/dev.hewig.JayJay.metainfo.xml
            install -Dm644 ${./docs/icon.svg} \
              $out/share/icons/hicolor/scalable/apps/dev.hewig.JayJay.svg
            for size in 64 128 256; do
              install -d "$out/share/icons/hicolor/''${size}x''${size}/apps"
              rsvg-convert \
                --width "$size" \
                --height "$size" \
                --output "$out/share/icons/hicolor/''${size}x''${size}/apps/dev.hewig.JayJay.png" \
                ${./docs/icon.svg}
            done
            wrapProgram $out/bin/jayjay-gpui \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeDeps}"
          '';

          # rpath so the AppImage finds the dlopen'd libs after bundling.
          postFixup = ''
            binary=$out/bin/jayjay-gpui
            if [ -x $out/bin/.jayjay-gpui-wrapped ]; then
              binary=$out/bin/.jayjay-gpui-wrapped
            fi
            patchelf --set-rpath "${pkgs.lib.makeLibraryPath runtimeDeps}" "$binary" || true
          '';
        });
      in
      {
        packages = {
          inherit jayjay-gpui;
          default = jayjay-gpui;
          appimage = nix-appimage.lib.${system}.mkAppImage {
            program = "${jayjay-gpui}/bin/jayjay-gpui";
            pname = "jayjay-gpui";
            name = "jayjay-gpui-${system}.AppImage";
          };
        };

        apps.default = flake-utils.lib.mkApp { drv = jayjay-gpui; };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ jayjay-gpui ];
          packages = with pkgs; [ rustc cargo clippy rustfmt ];
        };
      }
    );
}
