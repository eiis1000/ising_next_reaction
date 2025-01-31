use noisy_float::prelude::*;
use priority_queue::PriorityQueue;
use rand::prelude::*;
use rand_distr::Exp;
use std::cmp::Reverse;

use crate::ising_store::Ising;

pub struct TimeDistributions {
    pub distr: [Exp<f32>; Ising::NUM_ENERGIES as usize],
}

impl TimeDistributions {
    pub fn new(beta: f32) -> TimeDistributions {
        // from detailed balance, we know R(+dE)/R(-dE)=exp[-beta dE]
        // which gives us that R(dE)=r_0 exp[-beta dE / 2].
        // BUT... dE = -2E, so great, in total R(E -> -E) = r_0 exp[beta E]
        // we'll set r_0=1 to set the time scale
        let mut distr = [Exp::new(1.0).unwrap(); 9];
        for (i, d) in distr.iter_mut().enumerate() {
            *d = Exp::new(((i as i32 - 4) as f32 * beta).exp()).unwrap();
        }
        TimeDistributions { distr }
    }

    pub fn energy_to_index(energy: i8) -> usize {
        let nn = Ising::NUM_NEIGHBORS as i8;
        assert!(energy >= -nn && energy <= nn, "energy out of bounds");
        (energy + nn) as usize
    }

    pub fn direct_get(&self, index: usize) -> &Exp<f32> {
        &self.distr[index]
    }
}

pub struct TimeDistributionsResultBuffer<'a, R> {
    rng: &'a mut R,
    distr: &'a TimeDistributions,
    bufs: [Vec<f32>; Ising::NUM_ENERGIES],
    pub buf_sizes: [usize; Ising::NUM_ENERGIES],
}

impl<'a, R: Rng> TimeDistributionsResultBuffer<'a, R> {
    const DEFAULT_BUF_SIZE: usize = 100;
    const MAX_BUF_SIZE: usize = 100_000usize.next_power_of_two();

    pub fn new(
        distr: &'a TimeDistributions,
        rng: &'a mut R,
    ) -> TimeDistributionsResultBuffer<'a, R> {
        TimeDistributionsResultBuffer {
            rng,
            distr,
            bufs: Default::default(),
            buf_sizes: [Self::DEFAULT_BUF_SIZE; Ising::NUM_ENERGIES],
        }
    }

    pub fn sample(&mut self, energy: i8) -> f32 {
        let index = TimeDistributions::energy_to_index(energy);
        if self.bufs[index].is_empty() {
            // println!("Filling {index} buffer of size {}", self.buf_sizes[index]);
            self.bufs[index] = self
                .distr
                .direct_get(index)
                .sample_iter(&mut self.rng)
                .take(self.buf_sizes[index])
                .collect();
            self.buf_sizes[index] = std::cmp::min(2 * self.buf_sizes[index], Self::MAX_BUF_SIZE);
        }
        self.bufs[index].pop().unwrap()
    }
}

pub struct IsingEvolutionManager<'a, R: Rng> {
    ising: &'a mut Ising,
    time: f32,
    pq: PriorityQueue<usize, Reverse<R32>>,
    distr: TimeDistributions,
    rng: &'a mut R,
}

impl<'a, R: Rng> IsingEvolutionManager<'a, R> {
    pub fn new(ising: &'a mut Ising, beta: f32, rng: &'a mut R) -> IsingEvolutionManager<'a, R> {
        let distr = TimeDistributions::new(beta);
        let pq = PriorityQueue::<usize, Reverse<R32>>::new();
        let mut res = IsingEvolutionManager {
            ising,
            time: 0.,
            pq,
            distr,
            rng,
        };
        res.initialize_pq_buffered();
        res
    }

    fn initialize_pq_buffered(&mut self) {
        let mut vq: Vec<(usize, Reverse<R32>)> = Vec::new();
        let (_, _, n_cells) = self.ising.get_size();
        println!("Building buffers...");
        let mut rng_buffers = TimeDistributionsResultBuffer::new(&self.distr, self.rng);
        println!("Initializing taus...");
        for ix in 0..n_cells {
            let tau = rng_buffers.sample(self.ising.energy(ix));
            vq.push((ix, Reverse(r32(tau))));
        }
        println!("Calculating timescale...");
        println!(
            "Timescale is {}",
            vq.iter().map(|(_, r)| r.0.raw()).sum::<f32>() / vq.len() as f32
        );
        println!("Extending pq...");
        self.pq.extend(vq);
    }

    // fn _initialize_pq_unbuffered(
    //     ising: &Ising,
    //     rng: &mut impl RngCore,
    //     distr: &TimeDistributions,
    // ) -> PriorityQueue<usize, R32> {
    //     let mut pq = PriorityQueue::new();
    //     let mut vq: Vec<(usize, R32)> = Vec::new();
    //     let (_, _, n_cells) = ising.get_size();
    //     for ix in 0..n_cells {
    //         let tau = distr.get(ising.energy(ix)).sample(rng);
    //         vq.push((ix, r32(tau)));
    //     }
    //     // println!("Extending pq by {}...", vq.len());
    //     pq.extend(vq);
    //     pq
    // }

    pub fn evolve_ising_until<F>(&mut self, t_final: f32, mut callback: F) -> u32
    where
        F: FnMut(&Ising),
    {
        let mut rng_buffers = TimeDistributionsResultBuffer::new(&self.distr, self.rng);
        let mut ct: u32 = 0;
        while self.time < t_final {
            let (ix, tau) = self.pq.pop().unwrap();
            self.time = tau.0.raw();
            self.ising.flip_ix(ix);
            self.pq.push(
                ix,
                Reverse(r32(self.time + rng_buffers.sample(self.ising.energy(ix)))),
            );
            for nix in self.ising.neighbor_indices(ix).data {
                self.pq.change_priority(
                    &nix,
                    Reverse(r32(self.time + rng_buffers.sample(self.ising.energy(nix)))),
                );
            }
            ct += 1;
            callback(&self.ising);
        }
        ct
    }

    pub fn get_time(&self) -> f32 {
        self.time
    }

    pub fn get_ising(&self) -> &Ising {
        self.ising
    }
}
