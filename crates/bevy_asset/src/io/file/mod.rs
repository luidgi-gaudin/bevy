//! A backend for [`Asset`] storage in the local filesystem.
//!
//! It can watch for changed assets and hotload them if the `file_watcher` feature is enabled.
//!
//! [`Asset`]: crate::Asset

#[cfg(feature = "file_watcher")]
mod file_watcher;

#[cfg(feature = "multi_threaded")]
mod file_asset;
#[cfg(not(feature = "multi_threaded"))]
pub(crate) mod sync_file_asset;

#[cfg(feature = "file_watcher")]
pub use file_watcher::*;
use tracing::{debug, error};

use alloc::borrow::ToOwned;
use std::{
    env,
    path::{Path, PathBuf},
};

pub(crate) fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(manifest_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        let executable = env::current_exe().unwrap();
        #[cfg(target_os = "macos")]
        if let Some(resources) = macos_bundle_resources(&executable) {
            return resources;
        }
        executable.parent().map(ToOwned::to_owned).unwrap()
    }
}

/// Returns the `Contents/Resources` directory of the macOS application bundle containing
/// `executable`, if `executable` is the main executable of a bundle (`Name.app/Contents/MacOS/name`).
///
/// Apple requires the resources of an application to be stored there, and not next to the
/// executable: code signing treats every file in `Contents/MacOS` as code.
#[cfg(any(target_os = "macos", test))]
fn macos_bundle_resources(executable: &Path) -> Option<PathBuf> {
    let macos = executable.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    let is_bundle = macos.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle.extension()? == "app";
    is_bundle.then(|| contents.join("Resources"))
}

/// I/O implementation for the local filesystem.
///
/// This asset I/O is fully featured but it's not available on `android` and `wasm` targets.
pub struct FileAssetReader {
    root_path: PathBuf,
}

impl FileAssetReader {
    /// Creates a new `FileAssetIo` at a path relative to the executable's directory, optionally
    /// watching for changes.
    ///
    /// See `get_base_path` below.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root_path = Self::get_base_path().join(path.as_ref());
        debug!(
            "Asset Server using {} as its base path.",
            root_path.display()
        );
        Self { root_path }
    }

    /// Returns the base path of the assets directory, which is normally the executable's parent
    /// directory.
    ///
    /// This is, in order of priority:
    /// - the `BEVY_ASSET_ROOT` environment variable, if it is set,
    /// - the `CARGO_MANIFEST_DIR` environment variable, which is set when running with `cargo run`,
    /// - on macOS, the `Contents/Resources` directory of the application bundle, if the executable
    ///   is the main executable of a bundle (`Name.app/Contents/MacOS/name`),
    /// - the directory of the executable.
    ///
    /// To change the directory of the assets relative to this path, set
    /// [`AssetPlugin::file_path`][crate::AssetPlugin::file_path].
    pub fn get_base_path() -> PathBuf {
        get_base_path()
    }

    /// Returns the root directory where assets are loaded from.
    ///
    /// See `get_base_path`.
    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}

/// A writer for the local filesystem.
pub struct FileAssetWriter {
    root_path: PathBuf,
}

impl FileAssetWriter {
    /// Creates a new [`FileAssetWriter`] at a path relative to the executable's directory, optionally
    /// watching for changes.
    pub fn new<P: AsRef<Path> + core::fmt::Debug>(path: P, create_root: bool) -> Self {
        let root_path = get_base_path().join(path.as_ref());
        if create_root && let Err(e) = std::fs::create_dir_all(&root_path) {
            error!(
                "Failed to create root directory {} for file asset writer: {}",
                root_path.display(),
                e
            );
        }
        Self { root_path }
    }
}

#[cfg(test)]
mod tests {
    use super::macos_bundle_resources;
    use std::path::{Path, PathBuf};

    #[test]
    fn macos_bundle_resources_directory() {
        assert_eq!(
            macos_bundle_resources(Path::new(
                "/Applications/My Game.app/Contents/MacOS/my_game"
            )),
            Some(PathBuf::from(
                "/Applications/My Game.app/Contents/Resources"
            ))
        );
        // Executables that are not the main executable of a bundle.
        for executable in [
            "/usr/local/bin/my_game",
            "target/release/my_game",
            "my_game",
            "/Applications/My Game.app/Contents/Helpers/my_game",
            "/Applications/My Game/Contents/MacOS/my_game",
            "/Contents/MacOS/my_game",
        ] {
            assert_eq!(macos_bundle_resources(Path::new(executable)), None);
        }
    }
}
