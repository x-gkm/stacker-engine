use heapless::Vec;

use crate::{Coords, Orientation, PieceKind};

type Kicks = Vec<Coords, 6>;

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

pub fn kick_offset(piece: PieceKind, orientation: Orientation, rotate_cw: i32) -> Kicks {
    if rotate_cw % 2 == 0 && piece != PieceKind::I && piece != PieceKind::O {
        return flip_kick(orientation);
    }

    kick_offset_part(piece, orientation)
        .into_iter()
        .zip(kick_offset_part(piece, orientation.rotate_cw(rotate_cw)))
        .map(|((x1, y1), (x2, y2))| (x1 - x2, y1 - y2))
        .collect()
}

fn kick_offset_part(piece: PieceKind, orientation: Orientation) -> Kicks {
    if piece == PieceKind::O {
        return match orientation {
            Orientation::N => [(0, 0)],
            Orientation::E => [(0, -1)],
            Orientation::S => [(-1, -1)],
            Orientation::W => [(-1, 0)],
        }
        .into();
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

    offsets.into()
}

fn flip_kick(orientation: Orientation) -> Kicks {
    match orientation {
        Orientation::N => [(0, 0), (0, 1), (1, 1), (-1, 1), (1, 0), (-1, 0)],
        Orientation::E => [(0, 0), (0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)],
        Orientation::S => [(0, 0), (1, 0), (1, 2), (1, 1), (0, 2), (0, 1)],
        Orientation::W => [(0, 0), (-1, 0), (-1, 2), (-1, 1), (0, 2), (0, 1)],
    }
    .into()
}
