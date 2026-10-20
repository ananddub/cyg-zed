# CYG for Zed

Zed extension for `.cyg` schemas — the schema language for deterministic,
networked simulation code.

## Features

- Tree-sitter syntax highlighting, code outline, indentation, bracket matching,
  and comment toggling.
- `cyg-lsp` language server: diagnostics, completion, hover, go to definition,
  references, rename, document symbols, workspace symbols, and semantic tokens.
- The language server is downloaded automatically for your OS and architecture
  on first use. No .NET runtime is required.

## Install

Install **CYG** from the Zed extension gallery, or install this directory as a
dev extension via **zed: extensions → Install Dev Extension**.

The extension resolves the language server in this order:

1. A `cyg-lsp` executable on `PATH` (useful for development).
2. A previously downloaded server.
3. The latest [GitHub release](https://github.com/ananddub/cyg-zed/releases)
   asset for your platform.

Highlighting and outline work without the language server; diagnostics and
completions start once the server download finishes.

Supported platforms: Linux x64/arm64, macOS x64/arm64, Windows x64.

## Grammar

The Tree-sitter grammar lives in
[ananddub/tree-sitter-cyg](https://github.com/ananddub/tree-sitter-cyg).

## License

MIT
