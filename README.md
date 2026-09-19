# fen-at-ply

Given a game's move list and a ply number, print the FEN of the position
at that point.

PGN movetext is easy to read but hard to eyeball into a board. If you're
looking at an annotated game and want to know exactly what the position
looked like after move 23, or you want to hand a specific position to
another tool, you either count squares on paper or paste the game into a
GUI just to click forward. This does the one step in between: replay the
moves, stop where you asked, print the FEN.

## Usage

```
fen-at-ply <movetext-file> [ply]
```

`movetext-file` is a plain text file of SAN moves, with move numbers
written the way a PGN exporter writes them (`1. e4 e5 2. Nf3 ...`, number
and dot as their own token, separated by whitespace). `ply` is the
half-move count to stop at. Leave it off to play the whole file.

Given `examples/ruy-lopez.txt`:

```
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
```

```
$ fen-at-ply examples/ruy-lopez.txt 2
rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKB1R b KQkq - 1 2
$ fen-at-ply examples/ruy-lopez.txt
r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4
```

(build with `cargo build --release`, or `cargo run --` during development)

## How it works

The board is a plain 8x8 array. Applying a SAN token means parsing out
the piece letter, disambiguation, capture flag, destination square, and
promotion, then scanning the board for the one piece of that kind and
color that could legally reach the destination given normal piece
movement (sliding pieces check that the path is clear). If more than one
piece matches, or none do, the move is rejected instead of guessed at.

The parser trusts the input the way a real PGN export can be trusted:
it does not verify that a move is legal in the sense of not leaving the
mover's own king in check. It just needs to know which piece the
notation refers to.

## Known limitations

- Move legality with respect to check is not verified.
- Only single-game plain movetext is read; PGN tag pairs (`[Event "..."]`
  etc.) are not stripped, so a file with headers needs those lines
  removed first.

## License

MIT, see LICENSE.
