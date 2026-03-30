use crate::{Coords, Orientation, PieceKind};

pub fn piece_blocks(kind: PieceKind, orientation: Orientation) -> [Coords; 4] {
    match (kind, orientation) {
        (PieceKind::I, Orientation::N) => [(0, 0), (-1, 0), (1, 0), (2, 0)],
        (PieceKind::I, Orientation::E) => [(0, 0), (0, -2), (0, -1), (0, 1)],
        (PieceKind::I, Orientation::S) => [(0, 0), (-2, 0), (-1, 0), (1, 0)],
        (PieceKind::I, Orientation::W) => [(0, 0), (0, -1), (0, 1), (0, 2)],

        (PieceKind::J, Orientation::N) => [(0, 0), (-1, 0), (-1, 1), (1, 0)],
        (PieceKind::J, Orientation::E) => [(0, 0), (0, 1), (1, 1), (0, -1)],
        (PieceKind::J, Orientation::S) => [(0, 0), (-1, 0), (1, -1), (1, 0)],
        (PieceKind::J, Orientation::W) => [(0, 0), (0, 1), (-1, -1), (0, -1)],

        (PieceKind::L, Orientation::N) => [(0, 0), (-1, 0), (1, 1), (1, 0)],
        (PieceKind::L, Orientation::E) => [(0, 0), (0, 1), (1, -1), (0, -1)],
        (PieceKind::L, Orientation::S) => [(0, 0), (-1, 0), (-1, -1), (1, 0)],
        (PieceKind::L, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (0, -1)],

        (PieceKind::O, Orientation::N) => [(0, 0), (0, 1), (1, 1), (1, 0)],
        (PieceKind::O, Orientation::E) => [(0, 0), (0, -1), (1, -1), (1, 0)],
        (PieceKind::O, Orientation::S) => [(0, 0), (0, -1), (-1, -1), (-1, 0)],
        (PieceKind::O, Orientation::W) => [(0, 0), (0, 1), (-1, 1), (-1, 0)],

        (PieceKind::S, Orientation::N) => [(0, 0), (1, 1), (0, 1), (-1, 0)],
        (PieceKind::S, Orientation::E) => [(0, 0), (1, -1), (1, 0), (0, 1)],
        (PieceKind::S, Orientation::S) => [(0, 0), (1, 0), (0, -1), (-1, -1)],
        (PieceKind::S, Orientation::W) => [(0, 0), (0, -1), (-1, 0), (-1, 1)],

        (PieceKind::T, Orientation::N) => [(0, 0), (-1, 0), (0, 1), (1, 0)],
        (PieceKind::T, Orientation::E) => [(0, 0), (0, 1), (1, 0), (0, -1)],
        (PieceKind::T, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, 0)],
        (PieceKind::T, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (0, -1)],

        (PieceKind::Z, Orientation::N) => [(0, 0), (-1, 1), (0, 1), (1, 0)],
        (PieceKind::Z, Orientation::E) => [(0, 0), (1, 1), (1, 0), (0, -1)],
        (PieceKind::Z, Orientation::S) => [(0, 0), (-1, 0), (0, -1), (1, -1)],
        (PieceKind::Z, Orientation::W) => [(0, 0), (0, 1), (-1, 0), (-1, -1)],
    }
}

pub fn kick_offset(piece: PieceKind, orientation: Orientation, rotate_cw: i32, n: i32) -> Coords {
    let (x1, y1) = kick_offset_part(piece, orientation, n);
    let (x2, y2) = kick_offset_part(piece, orientation.rotate_cw(rotate_cw), n);

    (x1 - x2, y1 - y2)
}

fn kick_offset_part(piece: PieceKind, orientation: Orientation, n: i32) -> Coords {
    if piece == PieceKind::O {
        return match orientation {
            Orientation::N => (0, 0),
            Orientation::E => (0, -1),
            Orientation::S => (-1, -1),
            Orientation::W => (-1, 0),
        };
    }

    let offsets = match (piece, orientation) {
        (PieceKind::I, Orientation::N) => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        (PieceKind::I, Orientation::E) => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        (PieceKind::I, Orientation::S) => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        (PieceKind::I, Orientation::W) => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
        (_, Orientation::N) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::E) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (_, Orientation::S) => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        (_, Orientation::W) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    };

    offsets[n as usize]
}
