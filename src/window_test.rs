//! Manual smoke test: actually opens a real window and runs the event loop.
//!
//! This can't be a normal automated test — `run()` blocks on the OS event
//! loop until you close the window, and there's no display in CI. It's
//! `#[ignore]`d so `cargo test` skips it by default; run it by hand when
//! you want to eyeball that the window/GPU/render path still works:
//!
//!     cargo test --test window_smoke -- --ignored --nocapture
//!
//! Close the window (or Alt+F4 / Cmd+Q) to end the test.

#[test]
#[ignore = "opens a real window and blocks until closed; run manually"]
fn opens_a_window() {
    // Replace `space_game` with your actual crate name (see Cargo.toml).
    let result = space_game::run();

    assert!(
        result.is_ok(),
        "run() returned an error instead of exiting cleanly: {:?}",
        result.err()
    );
}