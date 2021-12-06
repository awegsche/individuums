use std::time::Instant;

use egui::{Color32, CtxRef, TextureId};
use sdl2::{VideoSubsystem, event::Event, rect::Rect, render::Canvas, sys::SDL_GLattr, video::{GLContext, gl_attr::GLAttr}};
use egui_sdl2_gl::{DpiScaling, EguiStateHandler, painter::Painter};
use sdl2::video::{SwapInterval, Window};

pub struct BoardWindow {
    pub event_pump: sdl2::EventPump,
    pub video_subsystem: VideoSubsystem,
    pub srgba: Vec<Color32>,
    pub tex_id: TextureId,
    pub canvas: Canvas<Window>,
}



pub fn setup_gui(window_width: u32, window_height: u32, board_width: u32, board_height: u32, frame_time: u32) -> Egui_Ctx {
    // SDL setup (TODO: refactor as soon as possible)
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_context_version(3, 2);


    let window = video_subsystem
        .window("Brains", window_width, window_height)
        .opengl().resizable().build().unwrap();

    let _ctx = window.gl_create_context().unwrap();
    //window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();


    let (mut painter, egui_state): (Painter, EguiStateHandler) = egui_sdl2_gl::with_sdl2(&window, DpiScaling::Default);
    let egui_ctx = egui::CtxRef::default();
    let event_pump: sdl2::EventPump = sdl_context.event_pump().unwrap();
    let kb = sdl_context.keyboard();
    let srgba: Vec<Color32> = (0..board_width*board_height).into_iter().map(|_| Color32::BLACK).collect();

    let tex_id = painter.new_user_texture((board_width as usize, board_height as usize), &srgba, false);


    Egui_Ctx { egui_state, egui_ctx, sdl_context, event_pump, video_subsystem, srgba, tex_id, painter, starttime: Instant::now(),
               window, gl_ctx: _ctx, frame_time }
}

pub struct Egui_Ctx {
    pub egui_state: EguiStateHandler,
    pub egui_ctx: CtxRef,
    pub sdl_context: sdl2::Sdl,
    pub event_pump: sdl2::EventPump,
    pub video_subsystem: VideoSubsystem,
    pub srgba: Vec<Color32>,
    pub tex_id: TextureId,
    pub starttime: Instant,
    pub painter: Painter,
    pub window: Window,
    pub gl_ctx: GLContext,
    pub frame_time: u32,
}

impl Egui_Ctx {
    pub fn begin_frame(&mut self) {
        self.egui_state.input.time = Some(self.starttime.elapsed().as_secs_f64());
        self.egui_ctx.begin_frame(self.egui_state.input.take());
    }

    pub fn update_texture(&mut self) {
        self.painter.update_user_texture_data(self.tex_id, &self.srgba);
    }

    pub fn end_frame(&mut self) -> bool {

        let (egui_output, paint_cmds) = self.egui_ctx.end_frame();
        self.egui_state.process_output(&egui_output);
        let paint_jobs = self.egui_ctx.tessellate(paint_cmds);
        self.painter.paint_jobs(None, paint_jobs, &self.egui_ctx.texture());

        self.window.gl_swap_window();
        if !egui_output.needs_repaint {
            if let Some(event) = self.event_pump.wait_event_timeout(self.frame_time) {
                match event {
                    Event::Quit{..} => {return false; },
                    _ => {
                        self.egui_state.process_input(&self.window, event, &mut self.painter);
                    }
                }
            }
        }

        true
    }
}
