# Building for macOS and Apple Silicon

Bevy renders with [Metal](https://developer.apple.com/metal/) on macOS and iOS, through
[wgpu](https://wgpu.rs). There is nothing to enable: the Metal backend is always compiled in on
Apple platforms, and it is the one used on every Mac, whether it has an Apple Silicon chip
(M1 and later) or an Intel CPU.

## Running your game

Install the Xcode Command Line Tools and [Rust](https://rustup.rs), then run your game as usual:

```sh
xcode-select --install
cargo run --release
```

On an Apple Silicon Mac, `rustup` installs the `aarch64-apple-darwin` toolchain, so your game is
compiled natively for the chip: it doesn't go through Rosetta.

Bevy logs the GPU it uses when it starts. On Apple Silicon, it looks like this:

```text
INFO bevy_render::renderer: AdapterInfo { name: "Apple M2", ..., backend: Metal }
```

To choose the backend explicitly, set the `WGPU_BACKEND` environment variable
(`WGPU_BACKEND=metal cargo run`), or set it in code:

```rust
use bevy::{
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: WgpuSettings {
                backends: Some(Backends::METAL),
                ..default()
            }
            .into(),
            ..default()
        }))
        .run();
}
```

## Building for a specific architecture

To build for Apple Silicon or Intel Macs from any Mac, add the corresponding target:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin  # Apple Silicon
cargo build --release --target x86_64-apple-darwin   # Intel
```

The executables are in `target/<target>/release/`. You can merge them into a universal executable,
which runs natively on both kinds of Mac:

```sh
lipo -create -output my_game \
    target/aarch64-apple-darwin/release/my_game \
    target/x86_64-apple-darwin/release/my_game
```

The oldest macOS version your game supports is set by the `MACOSX_DEPLOYMENT_TARGET` environment
variable at build time (for example `MACOSX_DEPLOYMENT_TARGET=12.0`), and defaults to the minimum
supported by Rust for the target.

## Distributing your game as an application

macOS applications are bundles: folders with a `.app` extension, and a precise layout:

```text
My Game.app/
└── Contents/
    ├── Info.plist        # describes the application
    ├── MacOS/
    │   └── my_game       # the executable
    └── Resources/
        ├── assets/       # the assets of your game
        └── AppIcon.icns  # optional
```

When your game runs from a bundle, Bevy loads its assets from `Contents/Resources`, where Apple
requires them to be: code signing treats every file in `Contents/MacOS` as code. When running with
`cargo run`, assets are loaded from your project directory instead, and the `BEVY_ASSET_ROOT`
environment variable overrides both.

This script builds a universal application bundle, with the assets of your game:

```sh
#!/bin/sh
set -e

NAME="My Game"                 # The name displayed by macOS.
BINARY="my_game"               # The name of your crate's binary.
IDENTIFIER="com.example.mygame" # A unique identifier, in reverse domain name notation.
VERSION="1.0.0"

rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

APP="target/$NAME.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/$BINARY" \
    "target/aarch64-apple-darwin/release/$BINARY" \
    "target/x86_64-apple-darwin/release/$BINARY"
cp -R assets "$APP/Contents/Resources/"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$NAME</string>
    <key>CFBundleExecutable</key>
    <string>$BINARY</string>
    <key>CFBundleIdentifier</key>
    <string>$IDENTIFIER</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Sign the application with an ad-hoc signature, so that it can run on this Mac.
codesign --force --sign - "$APP"
```

To only target Apple Silicon, build for `aarch64-apple-darwin` only, and copy its executable to
`Contents/MacOS` instead of using `lipo`. To add an icon, copy an `.icns` file to
`Contents/Resources` and add a `CFBundleIconFile` key with its name to `Info.plist`.

### Signing and notarization

Executables must be signed to run on Apple Silicon. The ad-hoc signature above is enough to run
the application on your own Mac, but macOS blocks applications downloaded from the internet unless
they are signed with a Developer ID certificate and notarized by Apple. With an
[Apple Developer](https://developer.apple.com/programs/) account:

```sh
codesign --force --options runtime --timestamp \
    --sign "Developer ID Application: Your Name (TEAMID)" "$APP"
ditto -c -k --keepParent "$APP" my_game.zip
xcrun notarytool submit my_game.zip --keychain-profile "notary-profile" --wait
xcrun stapler staple "$APP"
```

See Apple's
[notarization documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
to create the keychain profile used by `notarytool`.

## Measuring performance

- Set `MTL_HUD_ENABLED=1` to display the Metal performance HUD over your game, with its frame rate
  and GPU time.
- The Metal System Trace template of Instruments, which comes with Xcode, shows the work done by
  the CPU and the GPU for each frame.
- See [profiling.md](profiling.md) for the profiling tools supported by Bevy.

## iOS

Bevy also runs on iPhones and iPads with Metal. See the [mobile example](../examples/mobile) for
an Xcode project that builds a Bevy game for iOS.
