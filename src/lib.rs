mod window;
mod debug;
mod input;
mod gpu;
mod config;
pub use window::run;

#[cfg(test)]
mod tests {
    use super::*;

    /// Actually opens a real window and runs the event loop. Not something
    /// `cargo test` can assert on automatically (it blocks until you close
    /// the window, and there's no display in CI), so it's ignored by
    /// default. Run it by hand to eyeball that things still work:
    ///
    ///     cargo test --lib window_opens -- --ignored --nocapture
    ///
    /// Close the window to end the test.
    use serial_test::serial;
    #[test]
    #[serial]
    #[ignore = "opens a real window and blocks until closed; run manually"]
    fn window_opens() {
        let result = run();
        assert!(
            result.is_ok(),
            "run() returned an error instead of exiting cleanly: {:?}",
            result.err()
        );
    }
}