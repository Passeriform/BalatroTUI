# Contributing to Balatro TUI

Thank you for your interest in contributing to Balatro TUI! Your help is greatly appreciated. Please follow these guidelines to make the process smooth for everyone.

## How to Contribute

1. **Fork the repository** and create your branch from `master`.
2. **Clone your fork** and set up the project:

    ```sh
    git clone https://github.com/Passeriform/BalatroTUI.git
    cd BalatroTUI
    cargo build
    ```

3. **Make your changes** in a new branch:

    ```sh
    git checkout -b my-feature
    ```

4. **Test your changes**:

    ```sh
    cargo test
    ```

5. **Commit and push** your changes:

    ```sh
    git add .
    git commit -m "[Meta] Describe your change"
    git push origin my-feature
    ```

    > This repository doesn't use conventional commits. Consider formatting the commit message by category `[<category>] <message>`

6. **Open a Pull Request** on GitHub and describe your changes.

## Code Style

- Follow Rust's standard formatting: `cargo fmt`
- Run `cargo clippy` to catch common mistakes
- Write clear, concise commit messages

## Reporting Issues

If you find a bug or have a feature request, please open an issue with details and steps to reproduce.

## Code of Conduct

Be respectful and constructive. Harassment or abusive behavior will not be tolerated.

---

Thank you for helping make Balatro TUI better!
