mod board;
mod san;

use board::Board;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: fen-at-ply <movetext-file> [ply]");
        eprintln!("  movetext-file: a file with SAN moves, e.g. '1. e4 e5 2. Nf3 Nc6'");
        eprintln!("  ply: half-move to stop at (default: play the whole file)");
        process::exit(1);
    }

    let text = match fs::read_to_string(&args[1]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("could not read '{}': {}", args[1], e);
            process::exit(1);
        }
    };

    let ply_limit = args.get(2).and_then(|s| s.parse::<usize>().ok());
    let movetext = strip_tag_pairs(&text);
    let moves = tokenize(&movetext);
    let limit = ply_limit.unwrap_or(moves.len());

    let mut board = Board::start();
    for (i, mv) in moves.iter().enumerate() {
        if i >= limit {
            break;
        }
        if let Err(e) = san::apply_san(&mut board, mv) {
            eprintln!("move {} ('{}'): {}", i + 1, mv, e);
            process::exit(1);
        }
    }

    println!("{}", board.to_fen());
}

// A PGN tag pair is a whole line like `[Event "F.I.D.E. World Cup"]`. The
// quoted value can contain spaces, so these have to be dropped a line at a
// time rather than as whitespace-delimited tokens like the movetext is.
fn strip_tag_pairs(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('['))
        .collect::<Vec<_>>()
        .join("\n")
}

// Move numbers ("1.") and result markers are expected as separate
// whitespace-delimited tokens, matching standard PGN export formatting. A
// result marker ends the game it belongs to, so a file holding several
// games back to back (each with its own tag pairs and movetext) yields just
// the moves of the first one.
fn tokenize(text: &str) -> Vec<String> {
    let mut moves = Vec::new();
    for tok in text.split_whitespace() {
        if is_move_number(tok) {
            continue;
        }
        if matches!(tok, "1-0" | "0-1" | "1/2-1/2" | "*") {
            break;
        }
        moves.push(tok.to_string());
    }
    moves
}

fn is_move_number(tok: &str) -> bool {
    let trimmed = tok.trim_end_matches('.');
    !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit())
}
