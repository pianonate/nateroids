## nateroids

created to teach [natepiano](https://youtube.com/natepiano) how to code games, visualizations and
simulations in bevy using the awesome programming language, rust. i started
with [this tutorial](https://www.youtube.com/@ZymartuGames),
added [bevy_rapier3d](https://www.rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy) for physics as well as a
few other dependencies you can find in cargo.toml. the goal is to make this interesting, playable, beautiful and fun.

install rust (from https://www.rust-lang.org/tools/install)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone this project

```
git clone https://github.com/pianonate/nateroids
```

run it (first time will take a while)

```
cargo run
```

start playing! (gawd i like it that rust has such minimal rigamarole)

## Building WASM Target

you can run this natively or you can target wasm to run it in a browser.
you can use http-server (or something equivalent) to serve the wasm target locally. you can install it with npm
or use your own - i've tested the wasm target with http-server and chrome and this combination works. ymmv.

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name spaceship-game --out-dir target/wasm32 --target web target/wasm32-unknown-unknown/release/nateroids.wasm
http-server -c-1 -o ./
```

## Useful Links

- [Bevy Home](https://bevyengine.org/learn/)
- [Bevy CheatBook Overview](https://bevy-cheatbook.github.io/overview.html) also this.
- [Blender docs](https://docs.blender.org/manual/en/latest/)
- [Rapier physics docs](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy)
- [Tainted Coders](https://taintedcoders.com/) has a lot of inside info about Bevy game development
