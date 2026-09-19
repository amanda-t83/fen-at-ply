use crate::board::{Board, Color, Piece, PieceKind};

// Applies one SAN token (e.g. "Nbd7", "exd5", "e8=Q", "O-O") to the board.
// This trusts the input the way a real PGN export can be trusted: it finds
// the one piece that could reach the stated square and moves it, without
// re-deriving check legality.
pub fn apply_san(board: &mut Board, raw: &str) -> Result<(), String> {
    let token = raw.trim_end_matches(|c| matches!(c, '+' | '#' | '!' | '?'));
    let color = board.turn;

    if matches!(token, "O-O" | "0-0" | "O-O-O" | "0-0-0") {
        castle(board, color, token.len() == 3)?;
        finish_move(board, false, false);
        return Ok(());
    }

    let mut s = token;
    let mut promotion = None;
    if let Some(idx) = s.find('=') {
        let c = s[idx + 1..].chars().next().ok_or("missing promotion piece")?;
        promotion = Some(piece_kind_from_char(c)?);
        s = &s[..idx];
    }

    let mut chars: Vec<char> = s.chars().collect();
    let kind = if chars[0].is_ascii_uppercase() {
        let k = piece_kind_from_char(chars[0])?;
        chars.remove(0);
        k
    } else {
        PieceKind::Pawn
    };

    let is_capture = chars.contains(&'x');
    chars.retain(|&c| c != 'x');

    if chars.len() < 2 {
        return Err(format!("cannot parse move '{}'", raw));
    }
    let rank_char = chars.pop().unwrap();
    let file_char = chars.pop().unwrap();
    let dest = square(file_char, rank_char)?;

    let mut dis_file = None;
    let mut dis_rank = None;
    for c in chars {
        if c.is_ascii_alphabetic() {
            dis_file = Some((c as u8 - b'a') as usize);
        } else if c.is_ascii_digit() {
            dis_rank = Some((c as u8 - b'1') as usize);
        }
    }

    let source = find_source(board, kind, color, dest, dis_file, dis_rank, is_capture)
        .ok_or_else(|| format!("no unambiguous source found for '{}'", raw))?;

    move_piece(board, source, dest, kind, color, promotion, is_capture);
    Ok(())
}

fn piece_kind_from_char(c: char) -> Result<PieceKind, String> {
    match c {
        'N' => Ok(PieceKind::Knight),
        'B' => Ok(PieceKind::Bishop),
        'R' => Ok(PieceKind::Rook),
        'Q' => Ok(PieceKind::Queen),
        'K' => Ok(PieceKind::King),
        other => Err(format!("unknown piece letter '{}'", other)),
    }
}

fn square(file_char: char, rank_char: char) -> Result<(usize, usize), String> {
    if !('a'..='h').contains(&file_char) || !('1'..='8').contains(&rank_char) {
        return Err(format!("bad square '{}{}'", file_char, rank_char));
    }
    Ok(((file_char as u8 - b'a') as usize, (rank_char as u8 - b'1') as usize))
}

fn find_source(
    board: &Board,
    kind: PieceKind,
    color: Color,
    dest: (usize, usize),
    dis_file: Option<usize>,
    dis_rank: Option<usize>,
    is_capture: bool,
) -> Option<(usize, usize)> {
    let mut found = None;
    for rank in 0..8 {
        for file in 0..8 {
            match board.squares[rank][file] {
                Some(p) if p.kind == kind && p.color == color => {}
                _ => continue,
            }
            if let Some(df) = dis_file {
                if file != df {
                    continue;
                }
            }
            if let Some(dr) = dis_rank {
                if rank != dr {
                    continue;
                }
            }
            if can_reach(board, (file, rank), dest, kind, color, is_capture) {
                if found.is_some() {
                    return None;
                }
                found = Some((file, rank));
            }
        }
    }
    found
}

fn can_reach(
    board: &Board,
    from: (usize, usize),
    to: (usize, usize),
    kind: PieceKind,
    color: Color,
    is_capture: bool,
) -> bool {
    let (ff, fr) = (from.0 as i32, from.1 as i32);
    let (tf, tr) = (to.0 as i32, to.1 as i32);
    let df = tf - ff;
    let dr = tr - fr;

    match kind {
        PieceKind::Pawn => {
            let dir = if color == Color::White { 1 } else { -1 };
            let start_rank = if color == Color::White { 1 } else { 6 };
            if is_capture {
                dr == dir
                    && df.abs() == 1
                    && (board.squares[to.1][to.0].is_some() || Some(to) == board.en_passant)
            } else if df != 0 {
                false
            } else if dr == dir {
                board.squares[to.1][to.0].is_none()
            } else if dr == 2 * dir && fr == start_rank {
                let mid = (fr + dir) as usize;
                board.squares[mid][from.0].is_none() && board.squares[to.1][to.0].is_none()
            } else {
                false
            }
        }
        PieceKind::Knight => matches!((df.abs(), dr.abs()), (1, 2) | (2, 1)),
        PieceKind::King => df.abs() <= 1 && dr.abs() <= 1 && (df != 0 || dr != 0),
        PieceKind::Bishop => df.abs() == dr.abs() && df != 0 && path_clear(board, from, to),
        PieceKind::Rook => ((df == 0) != (dr == 0)) && path_clear(board, from, to),
        PieceKind::Queen => {
            (((df == 0) != (dr == 0)) || (df.abs() == dr.abs() && df != 0))
                && path_clear(board, from, to)
        }
    }
}

fn path_clear(board: &Board, from: (usize, usize), to: (usize, usize)) -> bool {
    let (ff, fr) = (from.0 as i32, from.1 as i32);
    let (tf, tr) = (to.0 as i32, to.1 as i32);
    let step_f = (tf - ff).signum();
    let step_r = (tr - fr).signum();
    let mut f = ff + step_f;
    let mut r = fr + step_r;
    while (f, r) != (tf, tr) {
        if board.squares[r as usize][f as usize].is_some() {
            return false;
        }
        f += step_f;
        r += step_r;
    }
    true
}

fn move_piece(
    board: &mut Board,
    from: (usize, usize),
    to: (usize, usize),
    kind: PieceKind,
    color: Color,
    promotion: Option<PieceKind>,
    is_capture: bool,
) {
    let mut captured = board.squares[to.1][to.0].take();
    if kind == PieceKind::Pawn && is_capture && captured.is_none() && Some(to) == board.en_passant {
        // the captured pawn sits beside the mover, not on the destination square
        captured = board.squares[from.1][to.0].take();
    }
    let final_kind = promotion.unwrap_or(kind);
    board.squares[to.1][to.0] = Some(Piece { kind: final_kind, color });
    board.squares[from.1][from.0] = None;

    if kind == PieceKind::King {
        clear_castle_rights(board, color);
    }
    if kind == PieceKind::Rook {
        strip_rook_right(board, from, color);
    }
    if let Some(c) = captured {
        if c.kind == PieceKind::Rook {
            strip_rook_right(board, to, c.color);
        }
    }

    board.en_passant = if kind == PieceKind::Pawn && (to.1 as i32 - from.1 as i32).abs() == 2 {
        Some((from.0, ((from.1 + to.1) / 2)))
    } else {
        None
    };

    finish_move(board, kind == PieceKind::Pawn, is_capture || captured.is_some());
}

fn clear_castle_rights(board: &mut Board, color: Color) {
    if color == Color::White {
        board.castle_wk = false;
        board.castle_wq = false;
    } else {
        board.castle_bk = false;
        board.castle_bq = false;
    }
}

fn strip_rook_right(board: &mut Board, square: (usize, usize), color: Color) {
    match (square, color) {
        ((0, 0), Color::White) => board.castle_wq = false,
        ((7, 0), Color::White) => board.castle_wk = false,
        ((0, 7), Color::Black) => board.castle_bq = false,
        ((7, 7), Color::Black) => board.castle_bk = false,
        _ => {}
    }
}

fn castle(board: &mut Board, color: Color, kingside: bool) -> Result<(), String> {
    let rank = if color == Color::White { 0 } else { 7 };
    let allowed = match (color, kingside) {
        (Color::White, true) => board.castle_wk,
        (Color::White, false) => board.castle_wq,
        (Color::Black, true) => board.castle_bk,
        (Color::Black, false) => board.castle_bq,
    };
    if !allowed {
        return Err("castling not available".to_string());
    }
    let king = Piece { kind: PieceKind::King, color };
    let rook = Piece { kind: PieceKind::Rook, color };
    if kingside {
        board.squares[rank][4] = None;
        board.squares[rank][7] = None;
        board.squares[rank][6] = Some(king);
        board.squares[rank][5] = Some(rook);
    } else {
        board.squares[rank][4] = None;
        board.squares[rank][0] = None;
        board.squares[rank][2] = Some(king);
        board.squares[rank][3] = Some(rook);
    }
    clear_castle_rights(board, color);
    board.en_passant = None;
    Ok(())
}

fn finish_move(board: &mut Board, is_pawn_move: bool, is_capture: bool) {
    if is_pawn_move || is_capture {
        board.halfmove = 0;
    } else {
        board.halfmove += 1;
    }
    if board.turn == Color::Black {
        board.fullmove += 1;
    }
    board.turn = board.turn.opposite();
}
