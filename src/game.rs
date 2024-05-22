pub struct Board {
    layers: [i32; 6],
    pieces: [i32; 6],
    player: i32,
}

impl Board {
    pub fn new() -> Board {
        return Board {
            layers: [0; 6],
            pieces: [2; 6],
            player: 0,
        };
    }

    fn get_view(&self, player: i32) -> i32 {
        let zro = self.layers[(player * 3) as usize] & !self.layers[((player ^ 1) * 3) as usize];
        let one = (zro | self.layers[(player * 3 + 1) as usize])
            & !self.layers[((player ^ 1) * 3 + 1) as usize];
        let two = one | self.layers[(player * 3 + 1) as usize];
		return two & 0b111111111;
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

    fn idx(&self, size: i32) -> usize {
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

pub struct Move {
    is_new: bool,
    size: i32,
    from: i32,
    to: i32,
}

pub struct GameBoard {
    history: Vec<Move>,
    b: Board,
}

impl GameBoard {
    pub fn do_move(&mut self, m: Move) {
        match m.is_new {
            true => self.b.do_new_move(m.size, m.to),
            false => self.b.do_board_move(m.size, m.from, m.to),
        }
    }

    pub fn undo_move(&mut self, m: Move) {
        match m.is_new {
            true => self.b.undo_new_move(m.size, m.to),
            false => self.b.undo_board_move(m.size, m.from, m.to),
        }
    }
}
