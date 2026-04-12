# solarust 🪐

A random solar system simulator that runs entirely in your terminal.

default
![Demo](demo.gif)

with shaded planets
![Demo Shades](demo-s.gif)

respekting terminal theme (example shows Catppuccin Mocha)
![Demo Theme](demo-t.gif)

---

## Features

- 3–8 randomly generated planets per system
- Planets orbit the sun at varying speeds (Kepler-inspired: inner planets faster)
- Elliptical orbits that adapt to your terminal's aspect ratio
- Rendered with Unicode half-block characters (`▀`) for smooth, pixel-style graphics
- Phosphor-glow trails via per-frame intensity decay
- Day/night shading: the side of each planet facing away from the sun is darkened
- Planet and orbit sizes scale with terminal dimensions
- Color theme support: built-in `dark`/`light` themes or `ansi` to use the terminal's own palette (Catppuccin, Dracula, Solarized, …)
- Intro and supernova outro animations
- Responds to terminal resize

## Requirements

- A terminal with true color (24-bit) support
- Rust toolchain (`cargo`) for building from source

## Installation

```bash
git clone https://github.com/the-unknown/solarust
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
solarust [OPTIONS]
```

| Option       | Description                                       |
| ------------ | ------------------------------------------------- |
| `-p <n>`     | Start with exactly `n` planets                    |
| `-s`         | Start with day/night shading enabled              |
| `-t <theme>` | Color theme: `dark` (default), `light`, or `ansi` |
| `-h`         | Show help and exit                                |

### `-t ansi`

Queries the terminal's ANSI color palette via OSC escape sequences and uses
those colors for the planets and orbits. This means the simulation automatically
matches any theme you have configured — Catppuccin, Dracula, Solarized, Gruvbox,
and so on. Falls back to `dark` if the terminal does not support the query.

**tmux users:** OSC passthrough must be enabled in your `tmux.conf` (requires tmux ≥ 3.3):

```
set -g allow-passthrough on
```

| Key | Action                   |
| --- | ------------------------ |
| `q` | Quit                     |
| `r` | Generate new system      |
| `s` | Toggle day/night shading |

## Building manually

```bash
cargo build --release
./target/release/solarust
```

## License

Apache-2.0
