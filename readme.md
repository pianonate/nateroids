## nateroids

this game has been crated to teach [natepiano](https://youtube.com/natepiano) how to code games, visualizations and simulations in bevy using the awesome programmin language, rust.

install rust (from https://www.rust-lang.org/tools/install)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone this project

```
git clone http://github.com/pianonate/nateroids
```

run it (first time will take a while)

```
cargo run
```

start playing! (gawd i like it that rust has such minimal rigamarole)



## Building WASM Target

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name spaceship-game --out-dir target/wasm32 --target web target/wasm32-unknown-unknown/release/nateroids.wasm
http-server -c-1 -o ./
```

## Useful Links

- [Bevy Home](https://bevyengine.org/learn/)
- [Bevy Cheatbook Overview](https://bevy-cheatbook.github.io/overview.html) also this.
- [Blender docs](https://docs.blender.org/manual/en/latest/)
- [Rapier physics docs](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy)
- [Tainted Coders](https://taintedcoders.com/) has a lot of inside info about Bevy game development