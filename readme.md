## nateroids

created to teach [natepiano](https://youtube.com/natepiano) how to code games, visualizations and
simulations in [bevy](https://bevyengine.org) using the awesome programming language, rust. i started
with [this tutorial](https://www.youtube.com/@ZymartuGames),
added [bevy_rapier3d](https://www.rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy) for physics as well as a
few other dependencies you can find in cargo.toml. the goal is to make this interesting, playable, beautiful and fun.

install rust (from https://www.rust-lang.org/tools/install)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone this project

```shell
git clone https://github.com/pianonate/nateroids
```

run it (first time will take a while)

```shell
cargo run
```

start playing! (gawd i like it that rust has such minimal rigamarole)

## release targets

you can build a release version locally this way:

```shell
cargo build --release
```

or run it

```shell
cargo run --release
```

or you can target wasm to run it in a browser.
you can use http-server (or something equivalent) to serve the wasm target locally. you can install http-server with npm
or use whatever server you prefer - i've tested the wasm target with http-server and chrome and this combination works.
ymmv.

```shell
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name nateroids --out-dir target/wasm32 --target web target/wasm32-unknown-unknown/release/nateroids.wasm
http-server -c-1 -o ./
```
### wasm32 issue

one of the dependencies which is only conditionally compile into dev builds is called bevy-inspector-egui. In turn,
bevy-inspector-egui depends on bevy-egui - and bevy-inspector-egui exposes a "manage-clipboard" feature from bevy-egui.
This "manage-clipboard feature causes an error
when we build with "--target wasm32-unknown-unknown".

The error is raised because "manage-clipboard" is considered an unsafe api. the RUSTFLAGS="
--cfg=web_sys_unstable_apis" suppresses this error but it's annoying. i need to figure out a solution. Especially given
that I use
conditional compilation to exclude the inspector code as it is only for dev. So the actual code is not included in any
kind of --release build.

With this RUSTFLAGS, it works for now. There's probably a cargo trick to disable the feature from the
dependency,
but it's beyond me right now without doing a whole log of rigamarole...would love to clean this up...also...someone saw the 
[same issue](https://github.com/jakobhellermann/bevy-inspector-egui/pull/210) but it's not yet pulled into the code base


## rustfmt

use the nightly rustfmt in order to pick up all the features in the rustfmt.toml - do this once:
```shell
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
```

then run this any time you want your code formatted - at least before committing
```shell
cargo +nightly fmt
```
why `unstable_features = true` doesn't work i don't know - it's also only available in the nightly...

## useful
- [bevy](https://bevyengine.org/learn/) - home to all things bevy
- [bevy cheatbook](https://bevy-cheatbook.github.io/overview.html) - super useful
- [blender](https://docs.blender.org/manual/en/latest/) - in case you want to mangle some meshes
- [rapier physics](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy) - so we don't have to code the physics
- [tainted coders](https://taintedcoders.com/) - has a lot of inside baseball about bevy
