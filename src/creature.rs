use rand::{Rng};

use crate::genes::{Genom, Nucl, Scorer};

// add a dummy type for point2. likely we won't ever need more than that, but for the case a more
// sophisticated type (like nalgebra::Point2) is needed, we have the option to typedef it in.
pub struct Point2 {
    pub x: i32,
    pub y: i32
}

impl Point2 {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
}

type Point = Point2;

// The creatures struct
//
pub struct Creatures<N: Nucl> {
    pub genoms: Vec<Genom<N, NullScorer>>,
    pub positions: Vec<Point>,
    pub mutation_coeff: usize,
}

fn make_positions<R: Rng>(n: usize, world_width: i32, world_height: i32, rng: &mut R) -> Vec<Point> {
    (0..n).map(|_| Point::new(
                rng.gen_range(0..world_width),
                rng.gen_range(0..world_height)
            )).collect()
}

impl<N: Nucl> Creatures<N> {
    /// Creates a new `Creatures` object.
    /// Inputs:
    ///
    /// * `n_genoms` - number of individuums (= number of genoms)
    /// * `n_neurons` - number of neurons per brain
    /// * `mutation_coeff` - controls the mutation rate.
    ///     `1/mutation_coeff` change of a mutation happening
    pub fn new<R: Rng>(n_genoms: usize, n_neurons: usize, mutation_coeff: usize, world_height: i32, world_width: i32, rng: &mut R) -> Self {

        Self {
            genoms: (0..n_genoms).map(|_| Genom::random(n_neurons, rng)).collect(),
            positions: make_positions(n_genoms,world_width, world_height, rng),
            mutation_coeff
        }
    }
}

pub trait NeuronNucl: Nucl {
    fn simulate<R: Rng>(creatures: &mut Creatures<Self>, rng: &mut R, width: i32, height: i32);

    fn simulate_end<R: Rng>(creatures: &mut Creatures<Self>, rng: &mut R, width: i32, height: i32);

    fn end_generation<R: Rng>(creatures: &mut Creatures<Self>, rng: &mut R, width: i32, height: i32) {
        let n = creatures.genoms.len();
        Self::simulate_end(creatures, rng, width, height);
        let mutation_n = rng.gen_range(0..creatures.mutation_coeff);
        if mutation_n < n {
            creatures.genoms[mutation_n].mutate(rng);
        }

        creatures.positions = make_positions(n, width, height, rng);
    }
}

/// ------------------------------------------------------------------------------------------------
/// --- generic dump -------------------------------------------------------------------------------
/// ------------------------------------------------------------------------------------------------
#[derive(Clone)]
pub struct NullScorer;

impl Scorer for NullScorer {
    fn score(&self) -> f32 {
        0.0
    }
}
