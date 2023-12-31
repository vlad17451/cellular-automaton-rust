```
cargo build --release --target wasm32-unknown-unknown
```

```
wasm-bindgen --no-typescript --target web     --out-dir ./out/     --out-name "cellular-automaton-rust"     ./target/wasm32-unknown-unknown/release/cellular-automaton-rust.wasm
```

```html
<!doctype html>
<html lang="en">

<body style="margin: 0px;">
  <script type="module">
    import init from './cellular-automaton-rust.js'

    init().catch((error) => {
      if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
      }
    });
  </script>
</body>

</html>

```