# terminal-snake
A terminal-based snake game made in Rust.

Game board size expands to fit the terminal at startup. (Does not change during execution.)

Use `w` `a` `s` `d` or arrow keys to navigate the board.

The goal is to survive as many apples as you can. Score will be displayed on a game over. Afterwards you have the option to quit or retry.

This project uses the `termion` crate to manipulate the terminal it is run in.

The game runs in a bordered play area at a fixed tick rate.
