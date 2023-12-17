import { defineConfig } from "vite"
import wasm from "vite-plugin-wasm"

export default defineConfig({
    plugins: [wasm()],
    build: {
        // the other option is to add vite-plugin-top-level-await, but I believe it is okay - https://caniuse.com/mdn-javascript_operators_await_top_level
        target: "es2022"
    }
})
