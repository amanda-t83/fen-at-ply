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
    let moves = tokenize(&text);
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

// Move numbers ("1.") and result markers are expected as separate
// whitespace-delimited tokens, matching standard PGN export formatting.
fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|tok| !is_move_number(tok))
        .filter(|tok| !matches!(*tok, "1-0" | "0-1" | "1/2-1/2" | "*"))
        .map(|s| s.to_string())
        .collect()
}

fn is_move_number(tok: &str) -> bool {
    let trimmed = tok.trim_end_matches('.');
    !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit())
}
