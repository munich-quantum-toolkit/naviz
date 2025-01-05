use core::str;
#[cfg(not(target_arch = "wasm32"))]
use std::{sync::mpsc::channel, thread};

use eframe::egui_wgpu::CallbackTrait;
use log::error;
use naviz_parser::config::{machine::MachineConfig, visual::VisualConfig};
use naviz_renderer::renderer::Renderer;
use naviz_repository::Repository;
use naviz_state::{config::Config, state::State};
#[cfg(not(target_arch = "wasm32"))]
use naviz_video::VideoExport;

use crate::{
    animator_adapter::{AnimatorAdapter, AnimatorState},
    aspect_panel::AspectPanel,
    canvas::{CanvasContent, EmptyCanvas, WgpuCanvas},
    current_machine::CurrentMachine,
    future_helper::FutureHelper,
    menu::{FileType, MenuBar, MenuConfig, MenuEvent},
    util::WEB,
};

/// The main App to draw using [egui]/[eframe]
pub struct App {
    future_helper: FutureHelper,
    menu_bar: MenuBar,
    animator_adapter: AnimatorAdapter,
    machine_repository: Repository,
    style_repository: Repository,
    current_machine: CurrentMachine,
}

impl App {
    /// Create a new instance of the [App]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        RendererAdapter::setup(cc);

        let mut machine_repository = Repository::empty()
            .bundled_machines()
            .expect("Failed to load bundled machines");
        let mut style_repository = Repository::empty()
            .bundled_styles()
            .expect("Failed to load bundled styles");

        // Load user-dirs only on non-web builds as there is no filesystem on web
        if !WEB {
            machine_repository = machine_repository
                .user_dir_machines()
                .expect("Failed to load machines from user dir");
            style_repository = style_repository
                .user_dir_styles()
                .expect("Failed to load styles from user dir");
        }

        let mut menu_bar = MenuBar::new();

        let mut animator_adapter = AnimatorAdapter::default();
        // Load any style as default (if any style is available)
        if let Some((id, style)) = style_repository.try_get_any() {
            animator_adapter.set_visual_config(style);
            menu_bar.set_selected_style(Some(id.to_string()));
        }
        // Load any machine as default (if any machine is available)
        if let Some((id, machine)) = machine_repository.try_get_any() {
            animator_adapter.set_machine_config(machine);
            menu_bar.set_selected_machine(Some(id.to_string()));
        }

        let mut app = Self {
            future_helper: FutureHelper::new().expect("Failed to create FutureHelper"),
            menu_bar,
            animator_adapter,
            machine_repository,
            style_repository,
            current_machine: Default::default(),
        };
        app.update_machines();
        app.update_styles();
        app
    }

    /// Update the machines displayed in the menu from the repository
    fn update_machines(&mut self) {
        self.menu_bar.update_machines(
            self.machine_repository
                .list()
                .into_iter()
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .collect(),
        );

        if let Some(instructions) = self.animator_adapter.get_instructions() {
            self.menu_bar
                .set_compatible_machines(&instructions.directives.targets);
        }
    }

    /// Update the styles displayed in the menu from the repository
    fn update_styles(&mut self) {
        self.menu_bar.update_styles(
            self.style_repository
                .list()
                .into_iter()
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .collect(),
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    // Update machine if not compatible or not set
                    if !input.directives.targets.is_empty()
                        && !self
                            .current_machine
                            .compatible_with(&input.directives.targets)
                    {
                        let compatible_machine = input
                            .directives
                            .targets
                            .iter()
                            .filter_map(|id| self.machine_repository.get(id).map(|m| (id, m)))
                            .next();
                        if let Some((id, compatible_machine)) = compatible_machine {
                            self.animator_adapter.set_machine_config(
                                compatible_machine.expect("Failed to load machine"),
                            );
                            self.current_machine = CurrentMachine::Id(id.clone());
                            self.menu_bar.set_selected_machine(Some(id.clone()));
                        }
                    }
                    self.menu_bar
                        .set_compatible_machines(&input.directives.targets);
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
                    self.current_machine = CurrentMachine::Manual;
                    self.menu_bar.set_selected_machine(None);
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
                    self.menu_bar.set_selected_style(None);
                }
                #[cfg(not(target_arch = "wasm32"))]
                MenuEvent::ExportVideo {
                    target,
                    resolution,
                    fps,
                    progress,
                } => {
                    if let Some(animator) = self.animator_adapter.animator() {
                        let video = VideoExport::new(animator, resolution, fps);
                        let (tx, rx) = channel();
                        self.future_helper.execute_to(video, tx);
                        thread::spawn(move || {
                            let mut video = rx.recv().unwrap();
                            video.export_video(&target, progress);
                        });
                    }
                }
                MenuEvent::SetMachine(id) => {
                    self.animator_adapter.set_machine_config(
                        self.machine_repository
                            .get(&id)
                            .expect("Invalid state: Selected machine does not exist")
                            .expect("Failed to load machine"),
                    );
                    self.current_machine = CurrentMachine::Id(id.clone());
                    self.menu_bar.set_selected_machine(Some(id));
                }
                MenuEvent::SetStyle(id) => {
                    self.animator_adapter.set_visual_config(
                        self.style_repository
                            .get(&id)
                            .expect("Invalid state: Selected style does not exist")
                            .expect("Failed to load style"),
                    );
                    self.menu_bar.set_selected_style(Some(id));
                }
                MenuEvent::ImportMachine(file) => {
                    self.machine_repository
                        .import_machine_to_user_dir(&file)
                        .expect("Failed to import machine");
                    self.update_machines();
                }
                MenuEvent::ImportStyle(file) => {
                    self.style_repository
                        .import_style_to_user_dir(&file)
                        .expect("Failed to import style");
                    self.update_styles();
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
            let padding = ui.style().spacing.item_spacing.y;
            let (_, space) = ui.allocate_space(ui.available_size());
            let panel = AspectPanel {
                space,
                aspect_ratio: 16. / 9.,
                top: 0.,
                bottom: 20. + padding,
                left: 0.,
                right: 0.,
            };
            let animator_state = self.animator_adapter.get();
            panel.draw(
                ui,
                |ui| {
                    if let Some(animator_state) = animator_state {
                        WgpuCanvas::new(RendererAdapter::new(animator_state)).draw(ctx, ui);
                    } else {
                        // Animator is not ready (something missing) => empty canvas
                        WgpuCanvas::new(EmptyCanvas::new()).draw(ctx, ui);
                    }
                },
                |_| {},
                |_| {},
                |ui| {
                    ui.add_space(padding);
                    self.animator_adapter.draw_progress_bar(ui);
                },
                |_| {},
            );
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
