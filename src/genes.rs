use rand::Rng;
use std::fmt::{Debug, Display, Formatter};

// --------------------------------------------------------------------------------------------
// --- Genetic Algorithm ----------------------------------------------------------------------
// --------------------------------------------------------------------------------------------

/// Nucleotide trait. A nucleotide has to provide the genetic operations Crossover and Mutate
/// as well as a possibility to instantiate it randomly.
///
/// Default is required because we are often constructing `Vec<N: Nucl>` with capacity.
/// Clone is required because we are constantly required to copy them around
/// We also require Debug to facilitate automatic debugging and error messages
pub trait Nucl: Debug + Clone + Default {

    /// crossover genetic operator. The child nucleotide is sharing values from both parents.
    /// In binary representation crossover of nucleotides might be just select the first one
    /// and the crossover of the Genom does all the work (i.e. selecting a crossover point)
    fn crossover(a: &Self, b: &Self) -> Self;

    /// mutation gentic operator. Randomly flip a bit (in bit representation) or change one value
    /// slightly
    fn mutate<R>(&mut self, rng: &mut R)
    where
        R: Rng;

    /// Nucleotides are in general randomly initialised
    fn random<R>(rng: &mut R) -> Self
    where
        R: Rng;
}

/// A Simulator is responsible for simulating the whole life of a genom.
/// It fills the genom with the fully calculated scorer.
pub trait Simu<N, S>: Debug
where
    N: Nucl,
    S: Scorer,
{
    fn simulate(&self, genom: &mut Genom<N,S>);
}

///
pub trait Scorer {
    fn score(&self) -> f32;
}


/// A Genom consists of a string of nucleotides
/// `ABBCDABABD`.
/// This struct provides various functions for initialisation
/// (from a given set of nucleotides, randomly, empty with capacity, from a previous generation).
///
/// Furthermore it implements the genetic operators, crossover and mutation
#[derive(Debug, Clone)]
pub struct Genom<N, S>
where
    N: Nucl,
    S: Scorer,
{
    pub nucleotides: Vec<N>,
    scorer: Option<S>,
}

impl<N,S> Genom<N,S>
where
    N: Nucl,
    S: Scorer
{
    // ---------------------------------------------------------------------------------------------
    // ---- Initialisation -------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// random initialisaton, this is the main way to initialise a new population
    pub fn random<R>(n: usize, rng: &mut R) -> Self
    where
        R: Rng,
    {
        Self {
            nucleotides: (0..n).map(|_| N::random(rng)).collect(),
            scorer: None,
        }
    }

    /// empty with capacity. This is mainly used if crossover_mut or a similar way is used to fill
    /// it after creation
    pub fn with_capacity(n: usize) -> Self {
        Genom {
            nucleotides: Vec::with_capacity(n),
            scorer: None,
        }
    }

    /// this just copies the nucleotides from a previous generation
    pub fn from_previous(other: &Self) -> Self {
        Genom {
            nucleotides: other.nucleotides.to_vec(),
            scorer: None,
        }
    }

    /// create a new genom from an iterator over nucleotides
    pub fn from_iter<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = N>,
    {
        Genom {
            nucleotides: iterator.into_iter().collect(),
            scorer: None,
        }
    }

    // ---------------------------------------------------------------------------------------------
    // ---Genetic operators ------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// mutatic crossover. this is meant to save time and resources of allocation (and hence speedup).
    /// I am not sure if this works
    ///
    /// For a general discussion of `crossover` please refer to [link]
    pub fn crossover_mut<R>(a: &Self, b: &Self, child: &mut Self, rng: &mut R)
    where
        R: rand::Rng,
    {
        let n = rng.gen_range(0..a.nucleotides.len());
        unsafe {
            child.nucleotides.set_len(0);
        }
        child.nucleotides.extend(
            a.nucleotides
                .iter()
                .zip(b.nucleotides.iter())
                .take(n)
                .map(|(first, second)| N::crossover(first, second)),
        );
        child.nucleotides.extend(
            a.nucleotides
                .iter()
                .zip(b.nucleotides.iter())
                .skip(n)
                .map(|(first, second)| N::crossover(second, first)),
        );
    }

    /// The crossover genetic operator
    /// The two genomes `AAAAAAAA` and `BBBBBBBB` give birth to a new genom
    /// `A'A'A'B'B'B'B'B`, where the crossover point is chosen randomly
    /// where `A' = crossover(A,B)` and `B' = crossover(B,A)`
    /// for non-binary representation
    pub fn crossover<R>(a: &Self, b: &Self, rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let n = rng.gen_range(0..a.nucleotides.len());
        let mut nucl: Vec<N> = Vec::with_capacity(a.nucleotides.len());
        nucl.extend(
            a.nucleotides
                .iter()
                .zip(b.nucleotides.iter())
                .take(n)
                .map(|(first, second)| N::crossover(first, second)),
        );
        nucl.extend(
            a.nucleotides
                .iter()
                .zip(b.nucleotides.iter())
                .skip(n)
                .map(|(first, second)| N::crossover(second, first)),
        );
        Genom {
            nucleotides: nucl,
            scorer: None,
        }
    }

    /// this is the more pure crossover function if the nucleotides are in binary representation.
    /// The child's genom is _actually_ `AAABBBBB`.
    pub fn crossover_cut<R>(a: &Self, b: &Self, rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let n = rng.gen_range(0..a.nucleotides.len());
        let mut nucl: Vec<N> = Vec::with_capacity(n);
        nucl.extend(a.nucleotides.iter().take(n).cloned());
        nucl.extend(b.nucleotides.iter().skip(n).cloned());
        Genom {
            nucleotides: nucl,
            scorer: None,
        }
    }

    /// this crossover function takes alternating quarters of both parents
    /// child = `AABBAABB`
    pub fn crossover_4th(a: &Self, b: &Self) -> Self {
        let n = a.nucleotides.len() / 4;
        let mut nucl: Vec<N> = Vec::with_capacity(n);
        nucl.extend(a.nucleotides.iter().take(n).cloned());
        nucl.extend(b.nucleotides.iter().skip(n).take(n).cloned());
        nucl.extend(a.nucleotides.iter().skip(2 * n).take(n).cloned());
        nucl.extend(b.nucleotides.iter().skip(3 * n).take(n).cloned());
        Genom {
            nucleotides: nucl,
            scorer: None,
        }
    }

    /// mutation randomly swaps a bit in the genom.
    /// For non-binary represantation this might be a just a small alteration to the value
    /// (+- a couple of per cent, where applicable)
    pub fn mutate<R>(&mut self, rng: &mut R)
    where
        R: rand::Rng,
    {
        let n = rng.gen_range(0..self.nucleotides.len());
        unsafe {
            self.nucleotides.get_unchecked_mut(n).mutate(rng);
        }
    }

    pub fn cut(&mut self, len: usize) {
        let mut new_nucl: Vec<N> = self
            .nucleotides
            .iter()
            .skip(len)
            .map(|x| x.clone())
            .collect();
        for _ in 0..len {
            new_nucl.push(N::default());
        }
        self.nucleotides = new_nucl;
    }

    // ---------------------------------------------------------------------------------------------
    // --- MISC ------------------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// A failsafe score property. Yields `0.0` if the scorer hasn't been calculated yet
    pub fn score(&self) -> f32 {
        if let Some(score) = &self.scorer {
            score.score()
        }
        else {
            0.0
        }
    }

    /// for dynamic usage, the foremost element might not be needed anymore, this shifts the nucleotides by 1
    pub fn shift(&mut self) {
        self.nucleotides.rotate_left(1);
    }

}

// The following is a convenience implementation for a Genom whose nucleotides implement Display

impl<N, S> Display for Genom<N, S>
where
    N: Nucl + Display,
S: Scorer
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if self.nucleotides.len() == 0 {
            return write!(f, "]");
        }

        write!(f, "{}", self.nucleotides[0])?;
        if self.nucleotides.len() > 30 {
            for nucl in self.nucleotides.iter().skip(1).take(10) {
                write!(f, ", {}", nucl)?;
            }
            write!(f, " ... {}", self.nucleotides[self.nucleotides.len() - 10])?;

            for nucl in self.nucleotides.iter().skip(self.nucleotides.len() - 9) {
                write!(f, ", {}", nucl)?;
            }
        }
        else {
            for nucl in self.nucleotides[1..].iter() {
                write!(f, ", {}", nucl)?;
            }
        }
        write!(f, "]")
    }
}
