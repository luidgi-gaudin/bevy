---
title: "Assets are loaded from `Contents/Resources` in macOS application bundles"
pull_requests: []
---

When a game runs from a macOS application bundle (its executable is `Name.app/Contents/MacOS/name`),
the default asset directory is now `Name.app/Contents/Resources/assets`, instead of
`Name.app/Contents/MacOS/assets`. Apple requires resources to be stored in `Contents/Resources`:
code signing treats every file in `Contents/MacOS` as code.

If you package your game in a bundle, move its `assets` folder from `Contents/MacOS` to
`Contents/Resources`. Alternatively, set the `BEVY_ASSET_ROOT` environment variable to the directory
containing your `assets` folder.

See [the macOS guide](https://github.com/bevyengine/bevy/blob/main/docs/macos.md) for how to build
and package a game for macOS.
