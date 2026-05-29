# Duck Hunt

A Duck Hunt clone built with [Bevy](https://bevyengine.org/), a game engine written in Rust.

---

## Prerequisites

### 1. Rust
Install Rust via [https://rustup.rs](https://rustup.rs). This installs both `rustc` (the compiler) and `cargo` (the package manager / build tool).

After installing, open a **new** terminal and verify:
```
rustc --version
cargo --version
```

### 2. C++ Build Tools (Windows only)
Rust on Windows requires the MSVC linker. Install **Visual Studio Build Tools**:

1. Download from [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/) → "Build Tools for Visual Studio"
2. Run the installer and select the **"Desktop development with C++"** workload
3. Click Install (~3–5 GB)

> macOS and Linux users do not need this step.

---

## Running the Game

```bash
cd duck-hunt
cargo run
```

The first run downloads and compiles all dependencies (Bevy + ~500 packages). This takes **5–10 minutes** on first build. Every run after that compiles only changed files and launches in a few seconds.

---

## How to Play

### Controls
| Action | Input |
|---|---|
| Aim | Move the mouse |
| Shoot | Left click |

The OS cursor is hidden — a red crosshair tracks your mouse instead.

### Objective
Shoot ducks as they fly across the screen. Survive as long as possible without letting 5 ducks escape.

### Rules
- Each wave gives you **3 shots** to hit the duck(s) on screen
- Shots reload when the screen clears between waves
- **5 total escaped ducks = Game Over** (tracked across all levels)
- Hit **6 out of 10 ducks** in a level to trigger **Level Up**
- You can still advance even if you miss some — only escapes count against your total

### Scoring
| Level | Points per duck |
|---|---|
| 1 | 100 |
| 2 | 150 |
| 3 | 200 |
| 4+ | +50 per level |

### Difficulty Scaling
Each level the ducks get faster and more ducks appear on screen simultaneously:

| Level | Max ducks on screen | Duck speed |
|---|---|---|
| 1 | 1 | 250 |
| 2 | 2 | 290 |
| 3+ | 3 | 330+ |

### HUD
| Display | Location | Description |
|---|---|---|
| `SCORE: X` | Top left | Running total score |
| `HITS: X / 6` | Top center | Ducks hit this level toward the 6-hit target |
| `LEVEL X` | Top right | Current level |
| `MISSES: X/5` | Bottom right | Total escaped ducks — reach 5 and it's game over |
| Bullet icons | Bottom left | Gold = loaded, dark = spent |

### Duck Varieties
Three duck types spawn randomly each wave — brown mallard, teal, and female/brown. Ducks fly in from the left edge, right edge, or bottom of the screen at random angles.

### The Dog
After each duck is shot or escapes, the dog pops up from the bushes:
- **Duck hit** — dog holds the duck up proudly with raised arms
- **Duck escaped** — dog laughs at you with squinting eyes and an open mouth

---

## Project Structure

```
duck-hunt/
├── src/
│   └── main.rs       # All game code (~700 lines)
├── Cargo.toml        # Dependencies (bevy 0.15, rand 0.8)
├── Cargo.lock        # Locked dependency versions
└── README.md
```

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [bevy](https://crates.io/crates/bevy) | 0.15 | Game engine (rendering, ECS, windowing, input) |
| [rand](https://crates.io/crates/rand) | 0.8 | Random duck spawn positions and angles |
