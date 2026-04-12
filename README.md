# solarust 🪐

A random solar system simulator that runs entirely in your terminal.

![Demo](demo.gif)

---

## Features

- 3–8 randomly generated planets per system
- Planets orbit the sun at varying speeds (Kepler-inspired: inner planets faster)
- Elliptical orbits that adapt to your terminal's aspect ratio
- Rendered with Unicode half-block characters (`▀`) for smooth, pixel-style graphics
- Phosphor-glow trails via per-frame intensity decay
- Intro and supernova outro animations
- Responds to terminal resize

## Requirements

- A terminal with true color (24-bit) support
- Rust toolchain (`cargo`) for building from source

## Installation

```bash
git clone https://github.com/yourname/solarust
cd solarust
make && make install
```

Installs to `~/.local/bin/solarust` by default. To install system-wide:

```bash
sudo make install PREFIX=/usr/local
```

To uninstall:

```bash
make uninstall
```

## Usage

```bash
solarust
```

| Key | Action              |
|-----|---------------------|
| `q` | Quit                |
| `r` | Generate new system |

## Building manually

```bash
cargo build --release
./target/release/solarust
```

## License

Apache-2.0
