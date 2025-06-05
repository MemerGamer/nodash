# nodash

`nodash` is a modern terminal-based project launcher and manager — fast, minimal, and built with Rust.

Easily manage and open your most-used dev projects with a slick TUI interface.

---

## ✨ Features

- ⚡ Launch projects with a single keypress
- 🗂 Store and manage your favorites
- 🎨 Beautiful TUI (powered by Ratatui)
- 📦 Self-updating via GitHub releases
- 🧩 Minimal dependencies, fast and portable

---

## 📥 Installation

To install the latest version:

```bash
curl -sSL https://raw.githubusercontent.com/MemerGamer/nodash/main/install.sh | bash
```

> Requires `curl` and `sudo` to move the binary to `/usr/local/bin`.

---

## 🚀 Usage

```bash
nodash
```

Interactive TUI will open.

### 🛠 Commands

```bash
nodash update
```

Check for updates and self-update the binary.

---

## 🔧 Building from Source

To build a release binary manually:

```bash
cargo build --release
```

This will generate the binary at:

```bash
target/release/nodash
```

You can move it to a directory in your `PATH`:

```bash
cp target/release/nodash /usr/local/bin/
```

---

## 📦 Releasing a New Version

To release a new version (e.g. `v1.2.3`):

1. Bump the version in `Cargo.toml`.
2. Commit and tag it:

   ```bash
   git commit -am "release: v1.2.3"
   git tag v1.2.3
   git push && git push --tags
   ```

3. Create GitHub release with binaries:

   - Build binaries for:

     - `linux-x86_64`
     - `linux-arm64`
     - `macos-x86_64`
     - `macos-arm64`

   - Name them like: `nodash-linux-x86_64`, etc.
   - Upload to the GitHub release.

---

## 🛡 License

MIT License © 2025 [MemerGamer](https://github.com/MemerGamer)

---

## 🙌 Contributing

PRs and feedback welcome! Feel free to [open an issue](https://github.com/MemerGamer/nodash/issues) or submit a pull request.
