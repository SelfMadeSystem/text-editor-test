# Text Editor Test

A little text editor I made with Rust and Speedy2D.

It uses a [piece table](https://en.wikipedia.org/wiki/Piece_table)-like data structure to store the text, which allows for fast inserts and deletes.

## Features

- [x] Fast inserts and deletes
- [ ] Undo/redo

## Controls

- Left/Right arrow keys: Move cursor
- Home/End: Move cursor to start/end of line
- Page Up: Move cursor to center of the file
- Page Down: Move cursor to end of the file
- Down: Print current file information to console

## Building

1. Install Rust and Cargo
2. Clone this repository
3. Run `cargo build --release` in the repository directory

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
