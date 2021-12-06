mod andis;
mod creature;
mod genes;
mod ui;
mod utils;
mod world;

use std::{ops::Add, time::Instant};

use egui::{vec2, Color32, Image, Layout};
use egui_sdl2_gl::DpiScaling;
use sdl2::{event::Event, pixels, rect::Rect, video::SwapInterval};

use crate::{
    andis::AndiN,
    ui::{setup_gui, Egui_Ctx},
    world::World,
};

// dimensions
const WWIDTH: u32 = 1600;
const WHEIGHT: u32 = 1024;
const BWIDTH: u32 = 512;
const BHEIGHT: u32 = 512;
const ZOOM: f32 = 2.0;

// fps
const FRAMETIME: u32 = 0;
const SIMS_PER_FRAME: u32 = 10;

// simulation
const N_CREATURES: usize = 1000;
const N_NEURONS: usize = 5;
const MUT_COEFF: usize = 1;
const STEPS_IN_GENERATION: u32 = 500;

// -------------------------------------------------------------------------------------------------

fn main() {
    println!("Hello, world!");

    let mut rng = rand::thread_rng();
    let mut world: World<_, AndiN> = World::new(
        N_CREATURES,
        N_NEURONS,
        MUT_COEFF,
        BWIDTH as i32,
        BHEIGHT as i32,
        rng,
    );
    world.set_steps_in_generation(STEPS_IN_GENERATION);

    let mut egui_ctx = setup_gui(WWIDTH, WHEIGHT, BWIDTH, BHEIGHT, FRAMETIME);

    let mut framecount = 0;
    let mut last_frametime = Instant::now();
    let mut fps = 0.0;
    let mut next_preview_frame = 0;

    'running: loop {
        egui_ctx.begin_frame();
        world.draw(&mut egui_ctx.srgba);
        egui_ctx.update_texture();

        egui::TopBottomPanel::top("hello").show(&egui_ctx.egui_ctx, |ui| {
            ui.label(format!("FPS: {:.2}", fps));
        });

        egui::SidePanel::right("details").show(&egui_ctx.egui_ctx, |ui| {
            world.details_ui(ui);
        });

        egui::CentralPanel::default().show(&egui_ctx.egui_ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.add(Image::new(
                        egui_ctx.tex_id,
                        vec2(BWIDTH as f32 * ZOOM, BHEIGHT as f32 * ZOOM),
                    ));
                    framecount += 1;
                },
            );
        });

        let elapsed = last_frametime.elapsed().as_secs_f32();
        if elapsed > 0.1 {
            fps = framecount as f32 / elapsed;
            last_frametime = Instant::now();
            framecount = 0;
        }

        if next_preview_frame == world.generation() {
            world.simulate();
        } else if next_preview_frame < world.generation() {
            next_preview_frame += SIMS_PER_FRAME;
            println!("end of generation. one random brain:");
            println!("{}", world.creatures.genoms[0]);
        } else {
            world.simulate_until_endofgeneration();
        }

        if !egui_ctx.end_frame() {
            break 'running;
        }
    }
}
