//! `cargo run -p aoa-client` opens a native window with the simulation in-process.
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    aoa_client::run_native();
}
