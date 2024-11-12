use core::str;
use std::{sync::mpsc::channel, thread};

use eframe::egui_wgpu::CallbackTrait;
use log::error;
use naviz_parser::config::{machine::MachineConfig, visual::VisualConfig};
use naviz_renderer::renderer::Renderer;
use naviz_state::{config::Config, state::State};
use naviz_video::VideoExport;

use crate::{
    animator_adapter::{AnimatorAdapter, AnimatorState},
    canvas::{CanvasContent, EmptyCanvas, WgpuCanvas},
    future_helper::FutureHelper,
    menu::{FileType, MenuBar, MenuConfig, MenuEvent},
};

/// The main App to draw using [egui]/[eframe]
pub struct App {
    future_helper: FutureHelper,
    menu_bar: MenuBar,
    animator_adapter: AnimatorAdapter,
}

impl App {
    /// Create a new instance of the [App]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        RendererAdapter::setup(cc);

        Self {
            future_helper: FutureHelper::new().expect("Failed to create FutureHelper"),
            menu_bar: MenuBar::new(),
            animator_adapter: AnimatorAdapter::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let time = ctx.input(|i| i.time);
        self.animator_adapter.set_time(time);

        // Check if a new file was read
        if let Ok(event) = self.menu_bar.events().try_recv() {
            match event {
                MenuEvent::FileOpen(FileType::Instructions, content) => {
                    let input = naviz_parser::input::lexer::lex(str::from_utf8(&content).unwrap())
                        .expect("Failed to lex");
                    let input =
                        naviz_parser::input::parser::parse(&input).expect("Failed to parse");
                    let input = naviz_parser::input::concrete::Instructions::new(input)
                        .expect("Failed to convert to instructions");
                    self.animator_adapter.set_instructions(input);
                }
                MenuEvent::FileOpen(FileType::Machine, content) => {
                    let machine =
                        naviz_parser::config::lexer::lex(str::from_utf8(&content).unwrap())
                            .expect("Failed to lex");
                    let machine =
                        naviz_parser::config::parser::parse(&machine).expect("Failed to parse");
                    let machine: naviz_parser::config::generic::Config = machine.into();
                    let machine: MachineConfig = machine
                        .try_into()
                        .expect("Failed to convert to machine-config");
                    self.animator_adapter.set_machine_config(machine);
                }
                MenuEvent::FileOpen(FileType::Style, content) => {
                    let visual =
                        naviz_parser::config::lexer::lex(str::from_utf8(&content).unwrap())
                            .expect("Failed to lex");
                    let visual =
                        naviz_parser::config::parser::parse(&visual).expect("Failed to parse");
                    let visual: naviz_parser::config::generic::Config = visual.into();
                    let visual: VisualConfig = visual
                        .try_into()
                        .expect("Failed to convert to visual-config");
                    self.animator_adapter.set_visual_config(visual);
                }
                MenuEvent::ExportVideo(target) => {
                    if let Some(animator) = self.animator_adapter.animator() {
                        let video = VideoExport::new(animator, (1920, 1080), 30);
                        let (tx, rx) = channel();
                        self.future_helper.execute_to(video, tx);
                        thread::spawn(move || {
                            let mut video = rx.recv().unwrap();
                            video.export_video(&target);
                        });
                    }
                }
            }
        }

        // Menu
        egui::TopBottomPanel::top("app_menu").show(ctx, |ui| {
            self.menu_bar.draw(
                MenuConfig {
                    export: self.animator_adapter.all_inputs_set(),
                },
                &self.future_helper,
                ctx,
                ui,
            )
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(animator_state) = self.animator_adapter.get() {
                WgpuCanvas::new(RendererAdapter::new(animator_state), 16. / 9.).draw(ctx, ui);
            } else {
                // Animator is not ready (something missing) => empty canvas
                WgpuCanvas::new(EmptyCanvas::new(), 16. / 9.).draw(ctx, ui);
            }
        });
    }
}

/// An adapter from [naviz_renderer] to [CallbackTrait].
///
/// Setup the renderer using [RendererAdapter::setup]
/// before drawing the renderer using the callback implementation.
#[derive(Clone)]
struct RendererAdapter {
    size: (u32, u32),
    /// The animator_state to render
    animator_state: AnimatorState,
}

impl RendererAdapter {
    /// Creates a [Renderer] and stores it in the egui [RenderState][eframe::egui_wgpu::RenderState].
    /// This created renderer will later be rendered from [RendererAdapter::paint].
    ///
    /// The renderer is stored in the renderer state
    /// in order for the graphics pipeline to have the same lifetime as the egui render pass.
    /// See [this section from the egui demo][https://github.com/emilk/egui/blob/0.28.1/crates/egui_demo_app/src/apps/custom3d_wgpu.rs#L83-L85]
    pub fn setup(cc: &eframe::CreationContext<'_>) {
        let wgpu_render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("No wgpu render state found");

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(Renderer::new(
                &wgpu_render_state.device,
                &wgpu_render_state.queue,
                wgpu_render_state.target_format,
                &Config::example(),
                &State::example(),
                (1920, 1080), // Use some default resolution to create renderer, as the canvas-resolution is not yet known
            ));
    }

    /// Creates a new [RendererAdapter] from the passed [AnimatorState]
    pub fn new(animator_state: AnimatorState) -> Self {
        Self {
            animator_state,
            size: Default::default(),
        }
    }
}

impl CallbackTrait for RendererAdapter {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(r) = callback_resources.get_mut::<Renderer>() {
            r.update_viewport(device, queue, self.size);
            self.animator_state
                .update(r, &mut (device, queue), device, queue);
        } else {
            error!("Failed to get renderer");
        }
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        if let Some(r) = callback_resources.get::<Renderer>() {
            r.draw(render_pass);
        } else {
            error!("Failed to get renderer");
        }
    }
}

impl CanvasContent for RendererAdapter {
    fn background_color(&self) -> egui::Color32 {
        let [r, g, b, a] = self.animator_state.background();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    fn target_size(&mut self, size: (f32, f32)) {
        self.size = (size.0 as u32, size.1 as u32);
    }
}
