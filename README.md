# fox flux advance

This is a port of [fox flux](https://github.com/eevee/fox-flux) to Rust and the Game Boy Advance.  It was started for a week-long jam, [GAMES MADE QUICK??? III](https://itch.io/jam/games-made-quick-iii), and may or may not continue development after that.

## Building

You will need:

- **Nightly** Rust, to actually build it.  Easiest way to get it is to install [`rustup`](https://rustup.rs/); the repository already has a file that will tell it to use nightly.
- [devkitPro](https://devkitpro.org/wiki/Getting_Started) on your path, for some miscellaneous ARM/GBA tooling.
- Python 3 with Pillow/PIL installed.  (If your `python` points to Python 2, you may need to edit the `Makefile` a bit.)

Then just run `make`!  Or `make release` for a release build, which is a good idea because debug builds are _a bit slow_ on the GBA.  Your finished ROM will be in `target/thumbv4-none-agb/{debug,release}/fox-flux-advance.gba`.  Run with [mGBA](https://mgba.io/), your favorite emulator, or your favorite flash cart, and enjoy.
