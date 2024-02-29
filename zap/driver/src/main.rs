// This is a temporary main file for testing.
// This crate will still exist, but will instead expose
// api to the cli crate and to wasm.

fn main() {
	driver::main("../net.zap".into(), None);
}
