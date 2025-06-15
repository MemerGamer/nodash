# nodash

![demo.png](demo.png)

`nodash` is a modern terminal-based project launcher and manager for nvm node projects — fast, minimal, and built with Rust.

Easily manage and open your most-used dev projects with a slick TUI interface.

---

## ✨ Features

- **Launch projects** with a single keypress, auto-loading Node.js version managers (nvm/fnm).
- **Project Management**: Add current directories, store and retrieve projects efficiently.
- **Node.js Version Display**: Shows the Node.js version specified in `.nvmrc` files.
- **Intelligent Sorting**: Projects are automatically sorted by their "last opened" date, with the most recent at the top.
- **Quick Search**: Filter projects instantly by name or path.
- **Clean TUI**: A modern, minimalist text-user interface designed for clarity and theme compatibility.
- **Self-Updating**: Keep `nodash` up-to-date directly from GitHub releases.
- **Minimal footprint**: Built with Rust for speed, efficiency, and portability.

---

## 📥 Installation

### General GNU/Linux

To install the latest version:

```bash
curl -sSL https://raw.githubusercontent.com/MemerGamer/nodash/main/install.sh | bash
```

> Requires `curl` and `sudo` to move the binary to `/usr/local/bin`.

### Arch Linux

```bash
yay nodash-bin
```

---

## 🚀 Usage

```bash
nodash
```

This command opens the interactive TUI dashboard.

### 🛠 Commands

You can also use `nodash` with direct commands:

```bash
nodash help
```

Display all available commands and interactive controls.

```bash
nodash add
```

Adds the current working directory as a new project in your `nodash` list. It will attempt to detect the Node.js version from a `.nvmrc` file if present.

```bash
nodash update
```

Check for updates and self-update the binary to the latest version.

### Interactive Controls (within the TUI)

- **↑/↓**: Navigate through the project list
- **Enter**: Open the selected project in a new terminal, automatically applying NVM/FNM version.
- **A**: Add a new project (prompts for name and path)
- **/**: Enter search mode to filter projects by name or path
- **Esc**: Exit search mode and clear the search query
- **Q**: Quit the application

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

You can move it to a directory in your `PATH` for easy access:

```bash
cp target/release/nodash /usr/local/bin/
```

---

## 🛡 License

MIT License © 2025 [MemerGamer](https://github.com/MemerGamer)

---

## 🙌 Contributing

PRs and feedback welcome! Feel free to [open an issue](https://github.com/MemerGamer/nodash/issues) or submit a pull request.
