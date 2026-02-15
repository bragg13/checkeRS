# checkeRS
A (PoC) Rust implementation of the board game checkers.

The game is not complete; in fact, it was intended only as a way to get familiar with Rust, TUIs (ratatui in this case), and Client-Server multiplayer gaming (with renet).

I took free inspiration from [this post by Herluf-ba](https://herluf-ba.github.io/making-a-turn-based-multiplayer-game-in-rust-01-whats-a-turn-based-game-anyway).

Roadmap:
- [x] TUI drawing
- [x] Checkboard grid, pawns with different colors
- [x] Pawn movement (simple, eating)
- [x] Client-server communication implementation
- [x] Main menu to select name and address
- [] Winning logic
- [] Pawn getting crowned
- [] Better scene management (eg. restart game, go to menu,...)

How to run:

1. First, you need to run the server
```bash
cargo run --bin server
```

2. Then, you can open up another terminal to launch the client
```bash
cargo run --bin client
```

From the main menu, you can selected your username and the address to connect to. There is no lobbying system for now, so each server can only handle one game at the moment.
