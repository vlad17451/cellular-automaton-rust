cargo build --features prod --no-default-features --release --target wasm32-unknown-unknown

wasm-bindgen --no-typescript --target web --out-dir ./build/ --out-name "cellular-automaton-rust" ./target/wasm32-unknown-unknown/release/cellular-automaton-rust.wasm

mkdir -p ./build

cp ./public/index.html ./build/index.html

cp -r ./assets ./build