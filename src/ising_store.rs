use bit_vec::BitVec;
use rand::prelude::*;
use std::fmt;

pub struct NeighborData<T> {
    pub data: [T; Ising::NUM_NEIGHBORS as usize],
}

impl<T> std::ops::Deref for NeighborData<T> {
    type Target = [T; Ising::NUM_NEIGHBORS as usize];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct Ising {
    store: BitVec,
    width: usize,
    height: usize,
    n_cells: usize,
}

impl Ising {
    pub const CHAR0: char = ' ';
    pub const CHAR1: char = '█';
    // when displaying the grid, prints each character twice for aspect ratio
    pub const REPEAT_CHARS: bool = true;
    // this is the number of neighbors of a given spin, not including itself
    pub const NUM_NEIGHBORS: u8 = 4;
    // if the i8 representation of the energy is between -m and +M, this is M+m+1
    pub const NUM_ENERGIES: u8 = Ising::NUM_NEIGHBORS * 2 + 1;

    pub fn new(width: usize, height: usize, rng: &mut impl Rng) -> Ising {
        let n_cells = width * height;
        assert_eq!(n_cells % 8, 0, "number of cells isn't a multiple of 8.");
        let n_bytes = n_cells / 8;

        println!("Allocating vec...");
        let mut bytes = vec![0u8; n_bytes];
        println!("Randomizing grid...");
        rng.fill_bytes(&mut bytes);
        println!("Building BitVec...");
        let store = BitVec::from_bytes(&bytes);

        Ising {
            store,
            width,
            height,
            n_cells,
        }
    }

    pub fn xy_to_ix(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    pub fn ix_to_xy(&self, ix: usize) -> (usize, usize) {
        (ix % self.width, ix / self.width)
    }

    pub fn get_size(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.n_cells)
    }

    pub fn neighbor_indices(&self, ix: usize) -> NeighborData<usize> {
        let (x, y) = self.ix_to_xy(ix);
        let left_ix = self.xy_to_ix((x + self.width - 1) % self.width, y);
        let right_ix = self.xy_to_ix((x + 1) % self.width, y);
        let up_ix = self.xy_to_ix(x, (y + self.height - 1) % self.height);
        let down_ix = self.xy_to_ix(x, (y + 1) % self.height);
        NeighborData {
            data: [left_ix, right_ix, up_ix, down_ix],
        }
    }

    // using i8 so that it can be more easily used in the energy calculation
    pub fn neighbor_states(&self, ix: usize) -> NeighborData<bool> {
        NeighborData {
            data: self.neighbor_indices(ix).map(|i| self.store[i]),
        }
    }

    // we have the luxury of an i8 type here because J=1
    pub fn energy(&self, ix: usize) -> i8 {
        let total_spin: i8 = self
            .neighbor_states(ix)
            .iter()
            .map(|&i| if i { 1i8 } else { -1i8 })
            .sum();
        if self.store[ix] {
            -total_spin
        } else {
            total_spin
        }
    }

    pub fn _get_ix(&self, ix: usize) -> bool {
        self.store[ix]
    }

    pub fn flip_ix(&mut self, ix: usize) {
        self.store.set(ix, !self.store[ix])
    }

    pub fn magnetization(&self) -> f64 {
        (self.store.count_ones() * 2) as f64 / (self.n_cells as f64) - 1.
    }
}

impl fmt::Display for Ising {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::with_capacity((self.width + 2) * self.height);
        let mut idx: usize = 0;
        for _ in 0..self.height {
            for _ in 0..self.width {
                let bit = self.store[idx];
                let c = if bit { Ising::CHAR1 } else { Ising::CHAR0 };
                output.push(c);
                if Ising::REPEAT_CHARS {
                    output.push(c);
                }
                idx += 1;
            }
            output.push('\n');
        }
        write!(f, "{}", output)
    }
}
