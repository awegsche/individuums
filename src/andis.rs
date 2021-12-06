use crate::{
    creature::{Creatures, NeuronNucl},
    genes::{Genom, Nucl, Scorer},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rand::Rng;
use std::fmt::{Debug, Display};

// -------------------------------------------------------------------------------------------------
// --- Andis Nucleotides ---------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------
#[derive(Default, Clone)]
#[repr(C)]
pub struct AndiN {
    encoded: u32,
}

impl Display for AndiN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[32m{:?}\x1b[0m", self.input())?;
        write!(f, " \x1b[90m{:.2}\x1b[0m", self.weight())?;
        write!(f, " \x1b[33m{:?}\x1b[0m", self.output())
    }
}

impl Debug for AndiN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[32m{:x}\x1b[0m", self.encoded >> 28)?;
        write!(f, "{:2x}", (self.encoded >> 12) & 0xFFFF)?;
        write!(f, "\x1b[90m{:02x}\x1b[0m", (self.encoded >> 4) & 0xFF)?;
        write!(f, "\x1b[33m{:x}\x1b[0m", self.encoded & 0x0_0000_00F)
    }
}

impl AndiN {
    pub fn decode(encoded: u32) -> Self {
        Self { encoded }
    }

    pub fn input(&self) -> InputNeurons {
        self.encoded.into()
    }

    pub fn weight(&self) -> f32 {
        ((self.encoded >> 8) & 0xFFFF) as f32 / 65536.0
    }

    pub fn output(&self) -> OutputNeurons {
        self.encoded.into()
    }
}

impl Nucl for AndiN {
    fn crossover(a: &Self, b: &Self) -> Self {
        a.clone()
    }

    fn mutate<R>(&mut self, rng: &mut R)
    where
        R: rand::Rng,
    {
        let bit = 2u32.pow(rng.gen_range(0..5));
        self.encoded += bit;
    }

    fn random<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Self {
            encoded: rng.next_u32(),
        }
    }
}

// the function
//
impl NeuronNucl for AndiN {
    fn simulate<R: Rng>(creatures: &mut Creatures<AndiN>, rng: &mut R, width: i32, height: i32) {
        // setup temp brain
        let n_neurons = OutputNeurons::COUNT as usize;
        let mut neurons: Vec<f32> = (0..n_neurons).into_iter().map(|_| 0.0).collect();
        let mut actions = Vec::with_capacity(creatures.genoms.len());

        for (i, (genom, pos)) in creatures
            .genoms
            .iter()
            .zip(creatures.positions.iter())
            .enumerate()
        {
            for nucl in genom.nucleotides.iter() {
                let signal = match nucl.input() {
                    InputNeurons::Osc => rng.gen_range(-1.0..1.0),
                    InputNeurons::PL => {
                        if pos.x < width / 2 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    InputNeurons::PR => {
                        if pos.x >= width / 2 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                } * nucl.weight();

                neurons[nucl.output() as usize] += signal;
            }

            for x in neurons.iter_mut() {
                *x = x.tanh();
            }

            let hor_motion =
                neurons[OutputNeurons::MvW as usize] - neurons[OutputNeurons::MvE as usize];
            let ver_motion =
                neurons[OutputNeurons::MvS as usize] - neurons[OutputNeurons::MvN as usize];

            if hor_motion > 0.5 {
                actions.push(Action::MoveEast(i));
            } else if hor_motion < -0.5 {
                actions.push(Action::MoveWest(i));
            }
            if ver_motion > 0.5 {
                actions.push(Action::MoveSouth(i));
            } else if hor_motion < -0.5 {
                actions.push(Action::MoveNorth(i));
            }

            // reset brain for next individuum
            for n in neurons.iter_mut() {
                *n = 0.0;
            }
        }

        for action in actions.iter() {
            match action {
                Action::MoveEast(i) => creatures.positions[*i].x += 1,
                Action::MoveWest(i) => creatures.positions[*i].x -= 1,
                Action::MoveNorth(i) => creatures.positions[*i].y += 1,
                Action::MoveSouth(i) => creatures.positions[*i].y -= 1,
            };
        }

        // sanitize positions
        for pos in creatures.positions.iter_mut() {
            if pos.x < 0 {
                pos.x = 0;
            }
            if pos.y < 0 {
                pos.y = 0;
            }
            if pos.x >= width {
                pos.x = width - 1;
            }
            if pos.y >= height {
                pos.y = height - 1;
            }
        }
    }

    fn simulate_end<R: Rng>(creatures: &mut Creatures<Self>, rng: &mut R, width: i32, height: i32) {
        let n = creatures.genoms.len();
        let parents: Vec<_> = creatures
            .genoms
            .iter()
            .zip(creatures.positions.iter())
            .filter(|(genom, pos)| pos.x > width / 2)
            .map(|(g, _)| g)
            .collect();
        println!("surviving parents: {}", parents.len());

        let mut partner: Vec<_> = parents
            .iter()
            .map(|p| (rng.next_u32(), p.clone()))
            .collect();
        partner.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_genoms: Vec<_> = parents
            .iter()
            .zip(partner.iter())
            .map(|(a, (_, b))| Genom::crossover(a, b, rng))
            .collect();

        while new_genoms.len() < n {
            for (x, _) in partner.iter_mut() {
                *x = rng.next_u32();
            }
            new_genoms.extend(parents
                .iter()
                .zip(partner.iter())
                .map(|(a, (_, b))| Genom::crossover(b, a, rng)));
        }

        creatures.genoms = new_genoms[..n].to_vec();
    }
}

// -------------------------------------------------------------------------------------------------
// --- Andis Scorer --------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------
//
pub struct AndiS {
    score_: f32,
}

impl Scorer for AndiS {
    fn score(&self) -> f32 {
        self.score_
    }
}

// -------------------------------------------------------------------------------------------------
// --- Input Neurons -------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

#[derive(FromPrimitive, Debug)]
pub enum InputNeurons {
    PR,
    PL,
    //WcF,
    Osc,
    COUNT,
}

impl From<u32> for InputNeurons {
    fn from(encoded: u32) -> Self {
        let byte = (encoded >> 24) as u8 % (InputNeurons::COUNT as u8);

        if let Some(neuron) = FromPrimitive::from_u8(byte) {
            neuron
        } else {
            InputNeurons::Osc
        }
    }
}

impl Into<u32> for InputNeurons {
    fn into(self) -> u32 {
        (self as u32) << 24
    }
}
// -------------------------------------------------------------------------------------------------
// --- Output Neurons ------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

#[derive(FromPrimitive, Debug)]
pub enum OutputNeurons {
    MvN,
    MvS,
    MvW,
    MvE,
    Wait,
    COUNT,
}

impl From<u32> for OutputNeurons {
    fn from(encoded: u32) -> Self {
        let byte = (encoded & 0xFF) as u8 % (OutputNeurons::COUNT as u8);

        if let Some(neuron) = FromPrimitive::from_u8(byte) {
            neuron
        } else {
            OutputNeurons::Wait
        }
    }
}

impl Into<u32> for OutputNeurons {
    fn into(self) -> u32 {
        self as u32
    }
}

// -------------------------------------------------------------------------------------------------
// --- Actions -------------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

enum Action {
    MoveEast(usize),
    MoveWest(usize),
    MoveNorth(usize),
    MoveSouth(usize),
}
