use egui::{vec2, Align, CollapsingHeader, Color32, Label, Layout, ScrollArea, TextStyle, Ui};
use rand::Rng;

use crate::{
    creature::{Creatures, NeuronNucl},
    genes::Nucl,
};

pub struct World<R: Rng, N: NeuronNucl> {
    pub creatures: Creatures<N>,
    grid: Vec<usize>,
    rng: R,
    width: i32,
    height: i32,
    step: u32,
    generation: u32,
    steps_in_generation: u32,
}

impl<R: Rng, N: NeuronNucl> World<R, N> {
    /// Creates a new world.
    /// Params:
    ///
    /// * `n_creatures` - number of creatures, each creature has its own genom
    /// * `n_neurons` - number of neurons per brain
    /// * `mutation_coeff` - mutation coefficient, a mutation happens with a probability `1/mutation_coeff`
    /// * `width` - the world width
    /// * `height` - the world height
    /// * `rng` - a suitable random number generator
    ///
    pub fn new(
        n_creatures: usize,
        n_neurons: usize,
        mutation_coeff: usize,
        width: i32,
        height: i32,
        mut rng: R,
    ) -> Self {
        Self {
            creatures: Creatures::new(
                n_creatures,
                n_neurons,
                mutation_coeff,
                height,
                width,
                &mut rng,
            ),
            grid: vec![n_creatures; (height * width) as usize],
            rng,
            width,
            height,
            step: 0,
            generation: 0,
            steps_in_generation: 300,
        }
    }

    // --- drawing ----------------------------------------------------------------------------------

    pub fn draw(&self, pixels: &mut [Color32]) {
        // reset to white
        for pixel in pixels.iter_mut() {
            *pixel = Color32::WHITE;
        }

        for position in self.creatures.positions.iter() {
            pixels[(position.x + self.width * position.y) as usize] = Color32::DARK_GREEN;
        }
    }

    pub fn details_ui(&self, ui: &mut Ui) {
        ui.add(
            Label::new("Simulation")
                .text_color(Color32::LIGHT_BLUE)
                .text_style(egui::TextStyle::Heading),
        );
        let th = vec2(
            ui.available_width(),
            ui.fonts()[TextStyle::Body].row_height() * 1.5,
        );
        let layout = Layout::left_to_right()
            .with_main_wrap(true)
            .with_cross_align(Align::BOTTOM);
        ui.allocate_ui_with_layout(th, layout, |ui| {
            ui.label("gen: ");
            ui.add(Label::new(format!("{}", self.generation)).strong());
            ui.label(", step: ");
            ui.add(Label::new(format!("{}", self.step)).strong());
            ui.end_row();

            ui.label("board: ");
            ui.add(Label::new(format!("{}", self.width)).strong());
            ui.label("x");
            ui.add(Label::new(format!("{}", self.height)).strong());
            ui.end_row();

            ui.label("steps in gen: ");
            ui.add(Label::new(format!("{}", self.steps_in_generation)).strong());
        });
        ui.separator();

        ui.add(
            Label::new("Individuums")
                .text_color(Color32::LIGHT_BLUE)
                .text_style(egui::TextStyle::Heading),
        );
        ScrollArea::vertical().show_rows(
            ui,
            ui.fonts()[TextStyle::Body].row_height(),
            self.creatures.positions.len(),
            |ui, range| {
                for pos in self.creatures.positions[range].iter() {
                    ui.label(format!("({}, {})", pos.x, pos.y));
                }
            },
        );
        ui.separator();
    }

    // --- simulation -------------------------------------------------------------------------------

    pub fn simulate_until_endofgeneration(&mut self) {
        while self.step < self.steps_in_generation {
            N::simulate(&mut self.creatures, &mut self.rng, self.width, self.height);
            self.step += 1;
        }
        self.step = 0;
        self.generation += 1;
        N::end_generation(&mut self.creatures, &mut self.rng, self.width, self.height);
    }

    pub fn simulate(&mut self) {
        N::simulate(&mut self.creatures, &mut self.rng, self.width, self.height);
        self.step += 1;

        if self.step > self.steps_in_generation {
            self.step = 0;
            self.generation += 1;
            N::end_generation(&mut self.creatures, &mut self.rng, self.width, self.height);
        }
    }

    // --- properties -------------------------------------------------------------------------------
    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn step(&self) -> u32 {
        self.step
    }

    pub fn set_steps_in_generation(&mut self, steps: u32) {
        self.steps_in_generation = steps;
    }
}
