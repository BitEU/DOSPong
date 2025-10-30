# Terminal Pong - Rust Edition

A high-performance, feature-rich Pong game for Windows conhost terminal with exciting power-ups!

## Features

- **Smooth rendering** - No screen flashing thanks to efficient buffer management
- **Optimized performance** - Runs smoothly even on low-end hardware
- **Power-ups:**
  - 🔷 **Double Paddle** - Get a second paddle for 10 seconds
  - 🟨 **Center Wall** - A full-height wall appears in the center
  - 🟪 **Two Small Walls** - Two partial walls block the center
  - 🟩 **Bent Paddle** - Your paddle becomes angled (<> shape)
  - ⚪ **Split Ball** - Current ball splits into three balls

## Controls

**Player 1 (Left - Blue):**
- `A` - Move up
- `D` - Move down

**Player 2 (Right - Red):**
- `Numpad 4` - Move up
- `Numpad 6` - Move down

**Game:**
- `ESC` or `Ctrl+Q` - Quit game

## Building

```bash
# Debug build (for testing)
cargo build

# Optimized release build (recommended)
cargo build --release
```

## Running

```bash
# Debug version
cargo run

# Release version (much faster)
cargo run --release
```

Or run the executable directly:
```bash
# After building
./target/release/terminal-pong.exe
```

## How to Play

1. The ball starts in the center and moves towards a random player
2. Hit the ball with your paddle to bounce it back
3. Power-ups appear randomly on the field - hit them with the ball to activate
4. Score points when your opponent misses the ball
5. First to... well, there's no limit! Play as long as you want!

## Power-up Details

- **Double Paddle (║)** - Cyan: Gives the player who hit it a second paddle below their main paddle
- **Center Wall (█)** - Yellow: Creates a full-height wall in the center that bounces the ball
- **Two Small Walls (▓)** - Magenta: Creates two partial walls that leave gaps for the ball to pass through
- **Bent Paddle (⟨)** - Green: Changes the paddle shape to an angled formation
- **Split Ball (✦)** - White: Immediately splits the ball into three separate balls

All power-ups (except Split Ball) last for 10 seconds.

## Performance Optimizations

- **Double buffering** - Entire frame is built in memory before rendering
- **Efficient rendering** - Only updates changed cells
- **Capped framerate** - Limited to ~60 FPS to prevent CPU waste
- **Zero-allocation gameplay** - No heap allocations during normal gameplay
- **LTO optimization** - Link-time optimization for maximum performance

## Tips

- The ball speeds up slightly each time it's hit
- Hit the ball at the edge of your paddle for more angle
- Power-ups are rare - use them strategically!
- Watch out for the bent paddle - it's wider but harder to aim with
- Multiple balls can be overwhelming - try to keep your paddle centered

Enjoy the game! 🏓