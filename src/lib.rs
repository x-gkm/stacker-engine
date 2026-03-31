#![no_std]

use heapless::Deque;
use serde::{Deserialize, Serialize};

use crate::random::PRNG;

mod random;
mod tables;

pub const PILE_HEIGHT: usize = 40;
pub const PILE_WIDTH: usize = 10;
pub const GRID_HEIGHT: i32 = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub spawn: u32,
    pub das: u32,
    pub arr: u32,
    pub are: u32,
    pub gravity: u32,
    pub softdrop: u32,
    pub line_clear: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            spawn: 60,
            das: 6,
            arr: 0,
            are: 0,
            gravity: 60,
            softdrop: 0,
            line_clear: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

pub type Coords = (i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    PieceKind(PieceKind),
    Garbage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    N,
    E,
    S,
    W,
}

impl Orientation {
    fn rotate_cw(&self, n: i32) -> Orientation {
        let mut result = *self;
        for _ in 0..n {
            use Orientation::*;
            result = match result {
                N => E,
                E => S,
                S => W,
                W => N,
            }
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Flip,
    Hold,
    Rotate(Direction),
    Move(Direction),
    Harddrop,
    Softdrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input {
    Begin(Action),
    End(Action),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn offset(&self) -> i32 {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct MovementState {
    das: Option<Direction>,
    move_left: bool,
    move_right: bool,
    soft_dropping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NextQueue {
    pieces: Deque<PieceKind, 11>,
    rng: PRNG,
}

impl NextQueue {
    fn new(seed: u64) -> NextQueue {
        let mut result = NextQueue {
            pieces: Deque::default(),
            rng: PRNG::new(seed),
        };

        result.add_bag();

        result
    }

    fn pull(&mut self) -> PieceKind {
        let result = self.pieces.pop_front().unwrap();
        if self.pieces.len() < 5 {
            self.add_bag();
        }
        result
    }

    fn add_bag(&mut self) {
        use PieceKind::*;
        let mut bag = [I, J, L, O, S, T, Z];
        self.rng.shuffle(&mut bag);
        self.pieces.extend(bag);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldPiece {
    pub kind: PieceKind,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Timer(u32);

impl Timer {
    fn tick(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }

        self.0 -= 1;

        if self.0 == 0 {
            return true;
        }

        false
    }

    fn set(&mut self, n: u32) {
        self.0 = n;
    }

    fn stop(&mut self) {
        self.0 = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FrameOutcome {
    pub tspin: bool,
    pub lines_cleared: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct BufferedInputs {
    hold: bool,
    rotation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Engine {
    config: Config,
    frame: i32,
    pile: Pile,
    active_piece: Option<Piece>,
    ghost_piece: Option<Piece>,
    hold: Option<HoldPiece>,
    next_queue: NextQueue,
    movement: MovementState,

    spawn_timer: Timer,
    fall_timer: Timer,
    das_timer: Timer,
    line_clear_timer: Timer,
    lock_timer: Timer,

    lowest_y: i32,
    resets: i32,

    game_over: bool,

    combo: Option<i32>,
    back_to_back: Option<i32>,

    frame_outcome: FrameOutcome,

    last_input_was_rotate: bool,

    buffered_inputs: BufferedInputs,

    garbage_rng: PRNG,
    garbage_queue: Deque<i32, 40>,
}

impl Engine {
    pub const FPS: i32 = 60;

    pub fn new(seed: u64, config: Config) -> Engine {
        let mut result = Engine {
            config,
            frame: 0,
            pile: Pile::default(),
            active_piece: None,
            ghost_piece: None,
            hold: None,
            next_queue: NextQueue::new(seed),
            movement: MovementState::default(),

            spawn_timer: Timer::default(),
            fall_timer: Timer::default(),
            das_timer: Timer::default(),
            line_clear_timer: Timer::default(),
            lock_timer: Timer::default(),

            lowest_y: 0,
            resets: 0,

            game_over: false,

            combo: None,
            back_to_back: None,

            frame_outcome: FrameOutcome::default(),

            last_input_was_rotate: false,

            buffered_inputs: BufferedInputs::default(),

            garbage_rng: PRNG::new(seed),
            garbage_queue: Deque::default(),
        };

        if result.config.spawn > 0 {
            result.spawn_timer.set(result.config.spawn);
        } else {
            result.spawn_next();
        }

        result
    }

    fn rotate(&mut self, count: i32) {
        let Some(ref active_piece) = self.active_piece else {
            self.buffered_inputs.rotation += count;
            return;
        };

        self.last_input_was_rotate = true;

        for (kick_x, kick_y) in
            tables::kick_offset(active_piece.kind, active_piece.orientation, count)
        {
            if let Some(piece) = self.can_move(kick_x, kick_y, count) {
                self.set_active(Some(piece));
                break;
            }
        }
    }

    fn lock_ghost(&mut self) {
        let Some(ref ghost_piece) = self.ghost_piece else {
            return;
        };

        if ghost_piece.lowest_y() >= GRID_HEIGHT {
            self.game_over = true;
            return;
        }

        for (x, y) in ghost_piece.blocks {
            self.pile.0[y as usize][x as usize] = Some(Cell::PieceKind(ghost_piece.kind))
        }
        let lines_to_clear = self.pile.lines_to_clear();

        if let Some(ref mut hold) = self.hold {
            hold.is_locked = false;
        }

        self.frame_outcome.lines_cleared = lines_to_clear;

        self.frame_outcome.tspin = self.last_input_was_rotate
            && ghost_piece.kind == PieceKind::T
            && self.pile.check_tspin(ghost_piece.x, ghost_piece.y);

        self.fall_timer.stop();
        self.set_active(None);
        let spawn_delay;
        if lines_to_clear > 0 {
            spawn_delay = if self.config.line_clear > self.config.are {
                self.config.line_clear
            } else {
                self.config.are
            };

            if self.config.line_clear > 0 {
                self.line_clear_timer.set(self.config.line_clear);
            } else {
                self.pile.line_clear();
            }

            if let Some(ref mut combo) = self.combo {
                *combo += 1;
            } else {
                self.combo = Some(0);
            }
            if lines_to_clear == 4 || (lines_to_clear > 0 && self.frame_outcome.tspin) {
                if let Some(ref mut back_to_back) = self.back_to_back {
                    *back_to_back += 1;
                } else {
                    self.back_to_back = Some(0);
                }
            } else {
                self.back_to_back = None;
            }
        } else {
            spawn_delay = self.config.are;

            self.combo = None;
        }

        if !self.garbage_queue.is_empty() {
            for &lines in &self.garbage_queue {
                let column = self.garbage_rng.random_range(0, 10);
                self.pile.push_garbage(lines, column);
            }

            self.garbage_queue.clear();
        }

        if spawn_delay > 0 {
            self.spawn_timer.set(spawn_delay);
        } else {
            self.spawn_next();
        }
    }

    fn can_move(&self, dx: i32, dy: i32, rotate_cw: i32) -> Option<Piece> {
        let Some(ref active_piece) = self.active_piece else {
            return None;
        };

        let branched = active_piece.changed_by(dx, dy, rotate_cw);
        if self.pile.check_collision(&branched.blocks) {
            return None;
        }

        Some(branched)
    }

    fn do_move(&mut self, direction: Direction) {
        let Some(piece) = self.can_move(direction.offset(), 0, 0) else {
            return;
        };

        self.last_input_was_rotate = false;

        self.set_active(Some(piece));
    }

    fn fall(&mut self) {
        let Some(piece) = self.can_move(0, -1, 0) else {
            return;
        };

        self.set_active(Some(piece));
    }

    fn try_lock(&mut self) {
        let Some(piece) = self.can_move(0, -1, 0) else {
            self.lock_ghost();
            return;
        };

        self.set_active(Some(piece));
    }

    fn set_fall_timer(&mut self) {
        let timeout = if self.movement.soft_dropping {
            self.config.softdrop
        } else {
            self.config.gravity
        };

        if timeout > 0 {
            self.fall_timer.set(timeout);
        } else {
            self.fall_timer.set(1);
            let Some(piece) = self.can_move(0, -1, 0) else {
                return;
            };

            self.set_active(Some(piece));
            self.set_fall_timer();
        }
    }

    fn spawn(&mut self, kind: PieceKind) {
        // It is very important to set resets to zero *before* calling set_active.
        self.resets = 0;
        self.set_active(Some(Piece::spawn(kind)));
        if self
            .pile
            .check_collision(&self.active_piece.as_ref().unwrap().blocks)
        {
            self.game_over = true;
            return;
        }
        self.lowest_y = self.active_piece.as_ref().unwrap().lowest_y();

        self.fall();

        if self.buffered_inputs.hold {
            self.buffered_inputs.hold = false;
            self.do_hold();
        }

        if self.buffered_inputs.rotation != 0 {
            self.rotate(self.buffered_inputs.rotation);
            self.buffered_inputs.rotation = 0;
        }

        self.set_fall_timer();
    }

    fn spawn_next(&mut self) {
        let piece = self.next_queue.pull();
        self.spawn(piece);
    }

    fn set_active(&mut self, piece: Option<Piece>) {
        let Some(piece) = piece else {
            self.active_piece = None;
            self.ghost_piece = None;
            return;
        };

        let previous_piece = self.active_piece.clone();
        let previous_lowest_y = self.lowest_y;

        self.active_piece = Some(piece);

        for dy in 0.. {
            let Some(branched) = self.can_move(0, -dy, 0) else {
                break;
            };
            self.ghost_piece = Some(branched);
        }

        self.lowest_y = self
            .lowest_y
            .min(self.active_piece.as_ref().unwrap().lowest_y());

        if self.lowest_y < previous_lowest_y {
            self.resets = 0;
        }

        if self.active_piece != previous_piece {
            if self.can_move(0, -1, 0) == None {
                self.lock_timer.set(30);
            } else {
                self.lock_timer.stop();
            }
        }

        self.resets += 1;

        if self.resets > 15 {
            self.try_lock();
        }
    }

    fn do_hold(&mut self) {
        let Some(ref mut active_piece) = self.active_piece else {
            self.buffered_inputs.hold = true;
            return;
        };

        let piece = match self.hold {
            Some(HoldPiece {
                is_locked: false,
                kind,
            }) => kind,
            None => self.next_queue.pull(),
            Some(HoldPiece {
                is_locked: true, ..
            }) => return,
        };

        self.hold = Some(HoldPiece {
            kind: active_piece.kind,
            is_locked: true,
        });
        self.fall_timer.stop();
        self.spawn(piece);
    }

    pub fn queue_garbage(&mut self, lines: i32) {
        // If we can't queue more garbage they are dead already so don't bother.
        let _ = self.garbage_queue.push_back(lines);
    }

    pub fn update(&mut self, frame_inputs: &[Input]) {
        self.frame_outcome = Default::default();

        if self.game_over {
            return;
        }

        self.frame += 1;

        // line_clear should be called before spawn so that the ghost piece isn't floating.
        if self.line_clear_timer.tick() {
            self.pile.line_clear();
        }
        if self.spawn_timer.tick() {
            self.spawn_next();
        }
        if self.fall_timer.tick() {
            self.fall();
            self.set_fall_timer();
        }
        if self.das_timer.tick() {
            let direction = self.movement.das.unwrap();
            self.do_move(direction);
            if self.config.arr > 0 {
                self.das_timer.set(self.config.arr);
            } else {
                self.das_timer.set(1);
                loop {
                    if let Some(piece) = self.can_move(direction.offset(), 0, 0) {
                        self.set_active(Some(piece));
                    } else {
                        break;
                    }
                }
            }
        }
        if self.lock_timer.tick() {
            self.try_lock();
        }

        for input in frame_inputs {
            use Action::*;
            use Direction::*;
            use Input::*;
            self.last_input_was_rotate = match input {
                Begin(Harddrop) | End(_) => self.last_input_was_rotate,
                _ => false,
            };
            match input {
                Begin(Rotate(Right)) => {
                    self.rotate(1);
                }
                Begin(Flip) => {
                    self.rotate(2);
                }
                Begin(Rotate(Left)) => {
                    self.rotate(3);
                }
                Begin(Hold) => {
                    self.do_hold();
                }
                Begin(Harddrop) => {
                    self.lock_ghost();
                }
                Begin(Move(Left)) => {
                    self.movement.move_left = true;
                    self.movement.das = Some(Left);
                    self.do_move(Left);
                    self.das_timer.set(self.config.das);
                }
                Begin(Move(Right)) => {
                    self.movement.move_right = true;
                    self.movement.das = Some(Right);
                    self.do_move(Right);
                    self.das_timer.set(self.config.das);
                }
                End(Move(Left)) => {
                    self.movement.move_left = false;
                    self.das_timer.stop();
                    if self.movement.move_right {
                        self.movement.das = Some(Direction::Right);
                        self.das_timer.set(self.config.das);
                    } else {
                        self.movement.das = None;
                    }
                }
                End(Move(Right)) => {
                    self.movement.move_right = false;
                    self.das_timer.stop();
                    if self.movement.move_left {
                        self.movement.das = Some(Direction::Left);
                        self.das_timer.set(self.config.das);
                    } else {
                        self.movement.das = None;
                    }
                }
                Begin(Softdrop) => {
                    self.fall();
                    self.movement.soft_dropping = true;
                    self.set_fall_timer();
                }
                End(Softdrop) => {
                    self.movement.soft_dropping = false;
                    self.set_fall_timer();
                }
                _ => (),
            }
        }
    }

    pub fn active_piece(&self) -> &Option<Piece> {
        &self.active_piece
    }

    pub fn ghost_piece(&self) -> &Option<Piece> {
        &self.ghost_piece
    }

    pub fn hold(&self) -> &Option<HoldPiece> {
        &self.hold
    }

    pub fn next_queue(&self) -> impl Iterator<Item = PieceKind> {
        self.next_queue.pieces.iter().take(5).copied()
    }

    pub fn pile(&self) -> &[[Option<Cell>; PILE_WIDTH]; PILE_HEIGHT] {
        &self.pile.0
    }

    pub fn combo(&self) -> i32 {
        self.combo.unwrap_or(0)
    }

    pub fn back_to_back(&self) -> i32 {
        self.back_to_back.unwrap_or(0)
    }

    pub fn frame_outcome(&self) -> &FrameOutcome {
        &self.frame_outcome
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn garbage_queue(&self) -> impl Iterator<Item = i32> {
        self.garbage_queue.iter().copied()
    }

    pub fn game_over(&self) -> bool {
        self.game_over
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Pile(#[serde(with = "serde_big_array::BigArray")] [[Option<Cell>; PILE_WIDTH]; PILE_HEIGHT]);

impl Default for Pile {
    fn default() -> Pile {
        Pile([[None; PILE_WIDTH]; PILE_HEIGHT])
    }
}

impl Pile {
    fn is_row_full(&self, row: usize) -> bool {
        self.0[row].iter().all(|&cell| cell != None)
    }

    fn lines_to_clear(&self) -> i32 {
        (0..PILE_HEIGHT)
            .filter(|&row| self.is_row_full(row))
            .count() as i32
    }

    fn line_clear(&mut self) {
        for row in (0..PILE_HEIGHT).rev() {
            if !self.is_row_full(row) {
                continue;
            }

            for ripple in row..PILE_HEIGHT - 1 {
                for cell in 0..PILE_WIDTH {
                    self.0[ripple][cell] = self.0[ripple + 1][cell];
                }
            }

            for cell in 0..PILE_WIDTH {
                self.0[PILE_HEIGHT - 1][cell] = None;
            }
        }
    }

    fn out_of_bounds(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= PILE_WIDTH as i32 || y < 0 || y >= PILE_HEIGHT as i32
    }

    fn has_block(&self, x: i32, y: i32) -> bool {
        self.0[y as usize][x as usize] != None
    }

    fn check_collision(&self, blocks: &[Coords]) -> bool {
        for &(x, y) in blocks {
            if self.out_of_bounds(x, y) || self.has_block(x, y) {
                return true;
            }
        }

        false
    }

    fn check_tspin(&self, x: i32, y: i32) -> bool {
        let offsets = [
            (x + 1, y + 1),
            (x + 1, y - 1),
            (x - 1, y + 1),
            (x - 1, y - 1),
        ];

        let mut count = 0;
        for (x, y) in offsets {
            if self.out_of_bounds(x, y) || self.has_block(x, y) {
                count += 1;
            }
        }

        count >= 3
    }

    fn push_garbage(&mut self, lines: i32, column: i32) {
        for y in (0..PILE_HEIGHT - lines as usize).rev() {
            for x in 0..PILE_WIDTH {
                self.0[y + lines as usize][x] = self.0[y][x]
            }
        }
        for y in 0..lines as usize {
            for x in 0..PILE_WIDTH {
                if x != column as usize {
                    self.0[y][x] = Some(Cell::Garbage);
                } else {
                    self.0[y][x] = None;
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub orientation: Orientation,
    pub x: i32,
    pub y: i32,
    pub blocks: [Coords; 4],
}

impl Piece {
    pub fn spawn(kind: PieceKind) -> Piece {
        let mut result = Piece {
            kind,
            orientation: Orientation::N,
            x: PILE_WIDTH as i32 / 2 - 1,
            y: GRID_HEIGHT + 2,
            blocks: Default::default(),
        };

        result.update_blocks();

        result
    }

    fn update_blocks(&mut self) {
        self.blocks = tables::piece_blocks(self.kind, self.orientation)
            .map(|(bx, by)| (self.x + bx, self.y + by));
    }

    fn changed_by(&self, dx: i32, dy: i32, rotate_cw: i32) -> Piece {
        let mut branched = self.clone();

        branched.x += dx;
        branched.y += dy;
        branched.orientation = branched.orientation.rotate_cw(rotate_cw);

        branched.update_blocks();

        branched
    }

    fn lowest_y(&self) -> i32 {
        self.blocks.map(|(_, y)| y).iter().copied().min().unwrap()
    }
}
