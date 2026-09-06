## nateroids

[![CI](https://github.com/natepiano/nateroids/actions/workflows/ci.yml/badge.svg)](https://github.com/natepiano/nateroids/actions/workflows/ci.yml)

created to teach [natepiano](https://youtube.com/natepiano) how to code games, visualizations and
simulations in [bevy](https://bevyengine.org) using rust. i started
with [this tutorial](https://www.youtube.com/@ZymartuGames),
added [avian3d](https://docs.rs/avian3d/latest/avian3d/) for physics as well as a
few other dependencies you can find in cargo.toml. the goal is to make this fun, playable, and beautiful.


## first install rust
install rust (from https://www.rust-lang.org/tools/install)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## make rust compile faster

For the edit/build/run loop, use `cargo dev` to run or `cargo dev-build` to build.
These project aliases enable incremental compilation even when your global Cargo
config disables it.

use sccache to make follow on compiles faster as it will cache locally anything you've already built. this comes in handy if you get other projects that all need to compile with bevy or 
anything you commonly depend on in these projects

```shell
cargo install sccache
```

and to enable it globally create / edit your $HOME/.cargo/config.toml by adding this to it - make sure you're using your local path - on my system it is:
```shell
[build]
rustc-wrapper = ".cargo/bin/sccache"
```

## clone nateroids project 

```shell
git clone https://github.com/natepiano/nateroids
```

run it - the first time will take a while even if you have sccache installed as you have to populate the cache, n'est-ce pas? This will build a debug version and run it:

```shell
cargo run
```

start playing! 

If you want to run it in release, you can do so by running:

```shell
cargo run --release
```

It might run faster on your machine. It will definitely be a smaller binary.

## Forwarded trackpad controls

To use a MacBook trackpad forwarded as mouse-wheel events through Deskflow:

```sh
NATEROIDS_FORWARDED_TRACKPAD=1 cargo run
```

Two-finger scrolling orbits, Shift+scroll pans, and Control+scroll zooms.
Physical mouse wheels receive those same controls while this mode is enabled.
Native trackpad input and middle-button dragging retain their existing controls.

Press Shift+C to open camera settings. Change `scroll_mode` between
`MouseWheel` and `ForwardedTrackpad`, or adjust `line_scroll_sensitivity`,
without restarting. Settings changed in the inspector last for the current run.
Without the environment variable, mouse-wheel scrolling zooms as before.
