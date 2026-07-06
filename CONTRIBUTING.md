# Contributing to LionFS

First off, thank you for considering contributing to LionFS. It's people like you that make LionFS such a great storage system.

## Where do I go from here?

If you've noticed a bug or have a feature request, make sure to check our [Issues](https://github.com/lionxlover/lionfs/issues) page to see if someone else in the community has already created a ticket. If not, go ahead and make one!

## Fork & create a branch

If this is something you think you can fix, then fork LionFS and create a branch with a descriptive name.

## Get the test suite running

Make sure your system has the prerequisites installed:
- Rust (latest stable)
- `pkg-config`
- `libfuse3-dev` (or `libfuse-dev` depending on the target backend)

To build and run tests:
```bash
cargo test
cargo build --all-targets
```

## Implement your fix or feature

At this point, you're ready to make your changes! Feel free to ask for help; everyone is a beginner at first 😸

## Code Style

- Use `cargo fmt` before committing.
- Ensure `cargo clippy` emits zero warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- Stick to safe Rust wherever possible. Unsafe code must be extensively commented and isolated to the lowest possible tier of the `disk` or `ondisk` hierarchy.

## Pull Request Process

1. Ensure any install or build dependencies are removed before the end of the layer when doing a build.
2. Update the README.md with details of changes to the interface, this includes new environment variables, exposed ports, useful file locations and container parameters.
3. Increase the version numbers in any examples files and the README.md to the new version that this Pull Request would represent.
4. The PR will be merged once you have the sign-off of at least one core maintainer.
