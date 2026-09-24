#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    fn fen_char(&self) -> char {
        let c = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        if self.color == Color::White {
            c.to_ascii_uppercase()
        } else {
            c
        }
    }
}

// squares[rank][file], rank 0 = "1", file 0 = "a". Coordinates elsewhere in
// this crate are (file, rank) tuples, matching how SAN reads (letter, digit).
#[derive(Clone)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub turn: Color,
    pub castle_wk: bool,
    pub castle_wq: bool,
    pub castle_bk: bool,
    pub castle_bq: bool,
    pub en_passant: Option<(usize, usize)>,
    pub halfmove: u32,
    pub fullmove: u32,
}

impl Board {
    pub fn start() -> Board {
        let mut squares = [[None; 8]; 8];
        let back = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        for file in 0..8 {
            squares[0][file] = Some(Piece { kind: back[file], color: Color::White });
            squares[1][file] = Some(Piece { kind: PieceKind::Pawn, color: Color::White });
            squares[6][file] = Some(Piece { kind: PieceKind::Pawn, color: Color::Black });
            squares[7][file] = Some(Piece { kind: back[file], color: Color::Black });
        }
        Board {
            squares,
            turn: Color::White,
            castle_wk: true,
            castle_wq: true,
            castle_bk: true,
            castle_bq: true,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    pub fn to_fen(&self) -> String {
        let mut ranks = Vec::with_capacity(8);
        for rank in (0..8).rev() {
            let mut row = String::new();
            let mut empty = 0;
            for file in 0..8 {
                match self.squares[rank][file] {
                    Some(p) => {
                        if empty > 0 {
                            row.push_str(&empty.to_string());
                            empty = 0;
                        }
                        row.push(p.fen_char());
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                row.push_str(&empty.to_string());
            }
            ranks.push(row);
        }
        let placement = ranks.join("/");

        let turn = if self.turn == Color::White { "w" } else { "b" };

        let mut castling = String::new();
        for (flag, ch) in [
            (self.castle_wk, 'K'),
            (self.castle_wq, 'Q'),
            (self.castle_bk, 'k'),
            (self.castle_bq, 'q'),
        ] {
            if flag {
                castling.push(ch);
            }
        }
        if castling.is_empty() {
            castling.push('-');
        }

        let ep = match self.en_passant {
            Some((file, rank)) => format!("{}{}", (b'a' + file as u8) as char, rank + 1),
            None => "-".to_string(),
        };

        format!(
            "{} {} {} {} {} {}",
            placement, turn, castling, ep, self.halfmove, self.fullmove
        )
    }
}
