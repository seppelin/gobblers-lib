pub mod search;

use std::fmt::{Debug, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub layers: [i32; 6],
    pub pieces: [i32; 6],
    pub player: i32,
}

impl Board {
    pub fn new() -> Board {
        return Board {
            layers: [0; 6],
            pieces: [2; 6],
            player: 0,
        };
    }

    pub fn is_winning_spot(self: &Board, pos: i32) -> bool {
        let view = self.get_view(self.player);
        let mut spots = 0;
        spots |= (view << 1) & (view << 2) & 0b100100100; // Right spots horizontal
        spots |= (view >> 1) & (view << 1) & 0b010010010; // Mid
        spots |= (view >> 2) & (view >> 1) & 0b001001001; // Left
        spots |= (view << 3) & (view << 6) & 0b111000000; // Top spots vertical
        spots |= (view >> 3) & (view << 3) & 0b000111000; // Mid
        spots |= (view >> 6) & (view >> 3) & 0b000000111; // Bot
        spots |= (view << 2) & (view << 4) & 0b000000100; // Top spots diag
        spots |= (view >> 2) & (view << 2) & 0b000010000; // Mid
        spots |= (view >> 4) & (view >> 2) & 0b001000000; // Bot
        spots |= (view << 4) & (view << 8) & 0b000000001; // Top spots diag
        spots |= (view >> 4) & (view << 4) & 0b000010000; // Mid
        spots |= (view >> 8) & (view >> 4) & 0b100000000; // Bot
        return spots & (1 << pos) != 0;
    }

    pub fn is_left(self: &Board, size: i32) -> bool {
        return self.pieces[self.idx(size)] > 0;
    }

    pub fn is_free(self: &Board, size: i32, pos: i32) -> bool {
        let same = match size {
            0 => {
                self.layers[0]
                    | self.layers[3]
                    | self.layers[1]
                    | self.layers[4]
                    | self.layers[2]
                    | self.layers[5]
            }
            1 => self.layers[1] | self.layers[4] | self.layers[2] | self.layers[5],
            2 => self.layers[2] | self.layers[5],
            _ => unreachable!(),
        };
        return ((1 << pos) & same) == 0;
    }

    pub fn is_movable(self: &Board, size: i32, pos: i32) -> bool {
        let bigger = match size {
            0 => self.layers[1] | self.layers[4] | self.layers[2] | self.layers[5],
            1 => self.layers[2] | self.layers[5],
            2 => 0,
            _ => unreachable!(),
        };
        return ((1 << pos) & self.layers[self.idx(size)] & !bigger) != 0;
    }

    pub fn get_view(&self, player: i32) -> i32 {
        let zro =
            self.layers[(player * 3) as usize] & !self.layers[((player ^ 1) * 3 + 1) as usize];
        let one = (zro | self.layers[(player * 3 + 1) as usize])
            & !self.layers[((player ^ 1) * 3 + 2) as usize];
        let two = one | self.layers[(player * 3 + 2) as usize];
        return two & 0b111111111;
    }

    pub fn is_line(view: i32) -> bool {
        let mut check = view & (view << 1) & (view << 2) & 0b100100100;
        check |= view & (view << 2) & (view << 4) & 0b001000000;
        check |= view & (view << 3) & (view << 6);
        check |= view & (view << 4) & (view << 8);
        return check != 0;
    }

    pub fn get_state(&self) -> i32 {
        let win = Self::is_line(self.get_view(self.player));
        let loss = Self::is_line(self.get_view(self.player ^ 1));
        return win as i32 | ((loss as i32) << 1);
    }

    pub fn is_cover(&self, size: i32, pos: i32) -> bool {
        let smaller = match size {
            0 => 0,
            1 => self.layers[0] | self.layers[3],
            2 => self.layers[1] | self.layers[4],
            _ => unreachable!(),
        };
        return ((1 << pos) & smaller) != 0;
    }

    pub fn idx(&self, size: i32) -> usize {
        return (self.player * 3 + size) as usize;
    }

    pub fn do_new_move(&mut self, size: i32, to: i32) {
        self.pieces[self.idx(size)] -= 1;
        self.layers[self.idx(size)] |= 1 << to;
        self.player ^= 1;
    }

    pub fn undo_new_move(&mut self, size: i32, to: i32) {
        self.player ^= 1;
        self.layers[self.idx(size)] ^= 1 << to;
        self.pieces[self.idx(size)] += 1;
    }

    pub fn do_board_move(&mut self, size: i32, from: i32, to: i32) {
        self.layers[self.idx(size)] ^= 1 << from;
        self.layers[self.idx(size)] |= 1 << to;
        self.player ^= 1;
    }

    pub fn undo_board_move(&mut self, size: i32, from: i32, to: i32) {
        self.player ^= 1;
        self.layers[self.idx(size)] ^= 1 << to;
        self.layers[self.idx(size)] |= 1 << from;
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.get_state();
        let mut s = String::new();
        s.write_str(&format!("Board p{} s{}\n", self.player, state))
            .unwrap();
        for size in (0..3).rev() {
            s.write_str("  ").unwrap();
            for pos in 0..9 {
                let mut c = '-';
                if self.layers[size] & (1 << pos) != 0 {
                    c = 'O';
                }
                if self.layers[size + 3] & (1 << pos) != 0 {
                    if c == 'O' {
                        c = '#';
                    } else {
                        c = 'X';
                    }
                }
                s.write_char(c).unwrap();
            }
            s.write_str(&format!(
                " | {}-{}\n",
                self.pieces[size],
                self.pieces[size + 3]
            ))
            .unwrap();
        }
        f.write_str(&s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    InGame,
    Win,
    Draw,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub is_new: bool,
    pub size: i32,
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Select {
    None,
    From,
    Move,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameBoard {
    history: Vec<Move>,
    b: Board,
    s: State,
    sel: Select,
    m: Move,
    pub auto_select: bool,
}

impl GameBoard {
    pub fn new(auto_select: bool) -> GameBoard {
        return GameBoard {
            history: Vec::new(),
            b: Board::new(),
            s: State::InGame,
            sel: Select::None,
            m: Move {
                is_new: false,
                size: 0,
                from: 0,
                to: 0,
            },
            auto_select,
        };
    }

    pub fn player(&self) -> i32 {
        return self.b.player;
    }

    pub fn get_board(&self) -> &Board {
        return &self.b;
    }

    fn get_id(&self) -> u64 {
        let mut id: u64 = 0;
        for i in 0..6 {
            id <<= 9;
            id |= self.b.layers[i] as u64;
        }
        id <<= 8;
        id |= self.history.len() as u64;
        id <<= 1;
        id |= self.b.player as u64;
        return id;
    }

    fn reorder(&mut self, order: [usize; 9]) {
        let mut tmp = [0; 6];
        for i in 0..6 {
            for pos in 0..9 {
                if self.b.layers[i] & (1 << pos) != 0 {
                    tmp[i] |= 1 << order[pos];
                }
            }
        }
        self.b.layers = tmp;
    }

    pub fn get_max_id(&self) -> u64 {
        let rotate = [2, 5, 8, 1, 4, 7, 0, 3, 6];
        let mirror = [6, 7, 8, 3, 4, 5, 0, 1, 2];
        let mut tmp = self.clone();
        let mut max_id = 0;
        for _ in 0..2 {
            for _ in 0..4 {
                max_id = self.get_id();
                tmp.reorder(rotate);
            }
            tmp.reorder(mirror);
        }
        return max_id;
    }

    pub fn get_state(&self) -> State {
        return self.s;
    }

    pub fn get_history(&self) -> &Vec<Move> {
        return &self.history;
    }

    pub fn get_left(&self, player: i32, size: i32) -> i32 {
        return self.b.pieces[(player*3 + size) as usize];
    }

    // Player + Size
    pub fn get_top(&self, pos: i32) -> Option<(i32, i32)> {
        for size in (0..3).rev() {
            for p in 0..2 {
                if self.b.layers[(p * 3 + size) as usize] & (1 << pos) != 0 {
                    return Some((p, size));
                }
            }
        }
        return None;
    }

    pub fn is_valid(&self, m: Move) -> bool {
        let from_ok = match m.is_new {
            true => self.b.is_left(m.size),
            false => self.b.is_movable(m.size, m.from),
        };
        return from_ok && self.b.is_free(m.size, m.to);
    }

    fn update_state(&mut self) {
        self.sel = Select::None;
        match self.b.get_state() {
            0 => self.s = State::InGame,
            1 => self.s = State::Win,
            2 => self.s = State::Loss,
            3 => self.s = State::Draw,
            _ => unreachable!(),
        }
    }

    pub fn select_board(&mut self, pos: i32) -> bool {
        if self.s != State::InGame {
            return false;
        }
        if self.sel == Select::None {
            if let Some((p, s)) = self.get_top(pos) {
                if p == self.b.player {
                    self.sel = Select::From;
                    self.m.is_new = false;
                    self.m.size = s;
                    self.m.from = pos;
                }
            }
        } else {
            if self.b.is_free(self.m.size, pos) {
                self.sel = Select::Move;
                self.m.to = pos;
                if self.auto_select {
                    self.submit_select();
                }
                return true;
            } else {
                self.sel = Select::None;
            }
        }
        return false;
    }

    pub fn select_new(&mut self, player: i32, size: i32) {
        if self.s != State::InGame {
            return;
        }
        if self.b.player == player && self.b.is_left(size) {
            self.sel = Select::From;
            self.m.is_new = true;
            self.m.size = size;
        } else {
            self.sel = Select::None;
        }
    }

    pub fn get_select(&self) -> (Select, Move) {
        return (self.sel, self.m);
    }

    pub fn is_selected_board(&self, pos: i32) -> bool {
        if self.sel != Select::None && !self.m.is_new {
            if self.m.from == pos {
                return true;
            }
        }
        if self.sel == Select::Move {
            if self.m.to == pos {
                return true;
            }
        }
        return false;
    }

    pub fn is_selected_new(&self, size: i32) -> bool {
        if self.sel != Select::None && self.m.is_new {
            if self.m.size == size {
                return true;
            }
        }
        return false;
    }    

    pub fn submit_select(&mut self) -> bool {
        if self.sel == Select::Move {
            self.do_move(self.m);
            return true;
        }
        return false;
    }

    pub fn remove_select(&mut self) {
        self.sel = Select::None;
    }

    pub fn do_move(&mut self, m: Move) -> bool {
        if !self.is_valid(m) || self.s != State::InGame {
            return false;
        }
        match m.is_new {
            true => self.b.do_new_move(m.size, m.to),
            false => self.b.do_board_move(m.size, m.from, m.to),
        };
        self.history.push(m);
        self.update_state();
        return true;
    }

    pub fn undo_move(&mut self) -> bool {
        let Some(m) = self.history.pop() else {
            return false;
        };
        match m.is_new {
            true => self.b.undo_new_move(m.size, m.to),
            false => self.b.undo_board_move(m.size, m.from, m.to),
        }
        self.update_state();
        return true;
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for to in 0..9 {
            for size in 0..3 {
                if !self.b.is_free(size, to) {
                    continue;
                }
                for from in 0..9 {
                    if !self.b.is_movable(size, from) {
                        continue;
                    }
                    moves.push(Move {
                        is_new: false,
                        size,
                        from,
                        to,
                    });
                }
                if !self.b.is_left(size) {
                    continue;
                }
                moves.push(Move {
                    is_new: true,
                    size,
                    from: -1,
                    to,
                });
            }
        }
        return moves;
    }
}
