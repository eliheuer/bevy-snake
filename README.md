# Bevy Snake Game

![Snake Game Screenshot](documentation/Screenshot-2025-01-10.png)

A classic Snake game implementation using Rust and the Bevy game engine.

## About

In this game, players control a snake that grows longer as it eats food while avoiding collisions with walls and itself. The original Snake game concept dates back to the 1976 arcade game Blockade, and it gained massive popularity in the late 1990s when it was pre-loaded on Nokia mobile phones and used as a demo application for Microsoft QBasic. [Read more about Snake's history on Wikipedia](https://en.wikipedia.org/wiki/Snake_(video_game)).

## Features

- Smooth snake movement using WASD or arrow keys
- Growing snake mechanics when eating food
- Score tracking
- Wall collision detection
- Self-collision detection
- Pause functionality (Press P)
- Game over state with restart option (Press SPACE)

## Building and Running

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Building

1. Clone the repository:

```bash
git clone https://github.com/eliheuer/bevy-snake.git
cd bevy-snake
```

2. Build and run the game:

```bash
cargo run
```

## How to Play

- Use WASD or arrow keys to move the snake.
- Eat the red food to grow.
- Don't hit walls or yourself.
- Press P to pause the game.
- Press SPACE to restart the game after it ends.

## Game Configuration

The game includes several configurable constants that can be adjusted in the source code `src/main.rs`:

## License

This project is free and open-source under the MIT License - see the LICENSE file for details.