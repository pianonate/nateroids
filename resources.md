## Useful Links

- [Bevy](https://bevyengine.org/learn/)
- [Bevy Windows Inside Baseball](https://taintedcoders.com/) has a lot of inside baseball about Bevy.
- [Bevy Cheatbook Overview](https://bevy-cheatbook.github.io/overview.html) also this.
- [Blender docs](https://docs.blender.org/manual/en/latest/)
- [Rapier docs](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy)

## Building WASM Target

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name spaceship-game --out-dir target/wasm32 --target web target/wasm32-unknown-unknown/release/spaceship_game.wasm
http-server -c-1 -o ./
