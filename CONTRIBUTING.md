# Contributing to Xylo

Thank you for your interest in contributing to Xylo! Whether you're improving the core language, adding examples, or fixing bugs, your work helps grow a creative coding tool for artists and programmers.

## Ways to Contribute

### 1. Reporting Issues

Found a bug or have a feature request? [Open an issue](https://github.com/giraffekey/xylo/issues/new) with:

- A clear title.
- Steps to reproduce the problem.
- Expected vs. actual behavior (include screenshots if it's a visual bug).
- Your environment (OS, Xylo version/build, Rust version, etc.).

### 2. Improving Documentation

Help make Xylo more accessible by:

- Fixing typos/clarifying the [GitBook docs](https://github.com/giraffekey/xylo-docs/).
- Adding tutorials (e.g., "How to create L-systems in Xylo").
- Documenting edge cases or compiler behaviors.

### 3. Setting Up for Development

#### Prerequisites

- Git
- Rust and Cargo (latest stable version).

#### Clone and Build

```sh
git clone https://github.com/giraffekey/xylo.git
cd xylo
cargo build            # Debug build
cargo build --release  # Release build
```

#### Run Tests

```sh
cargo test    # Unit tests (std)
cargo test --no-default-features --features no-std    # Unit tests (no_std)
cargo run --release -- generate example.xylo --width 800 --height 800  # Run the example
```

### 4. Making Changes

#### Code Structure

- **`src/`**: Interpreter core (parser, interpreter, renderer).
- **`src/functions/`**: Standard library functions

#### Workflow

1. **Branch off `main`**:
   ```sh
   git checkout -b your-branch
   ```
2. **Follow Rust conventions**:
   - Use `snake_case` for variables/functions.
   - Document public APIs with `///` comments.
   - Format with `cargo fmt` and lint with `cargo clippy`.
3. **Test rigorously**:
   - Add unit tests in `src/`.
   - Verify generative art outputs.

#### Commit Messages

Commit messages should be descriptive and finish the sentence "This commit will...".

Examples:
```
Add HSL color utilities
Handle division by zero errors
Add a function for creating fractals
```

### 5. Submitting a Pull Request

1. **Push your branch**:
   ```sh
   git push origin your-branch
   ```
2. **Open a PR**:
   - Target the `main` branch.
   - Describe changes and reference related issues (e.g., `Fixes #42`).
   - Include screenshots for changes that involve art generation.
3. **Address feedback**: Maintainers will review within 48 hours.

## Contribution Areas

### A. Core Interpreter

- Optimize performance and resource usage.
- Extend the type system or syntax.
- Write extensive tests for edge cases.

### B. Standard Library

- Add built-in utility functions.
- Improve error messages and handling.

### C. Tooling

- WASM/web playground.
- IDE plugins (VS Code, IntelliJ).

## Code of Conduct

Please review our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.
