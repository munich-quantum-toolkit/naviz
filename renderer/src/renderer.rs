use naviz_state::{config::Config, state::State};
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    buffer_updater::BufferUpdater,
    component::{atoms::Atoms, legend::Legend, machine::Machine, time::Time, ComponentInit},
    globals::Globals,
    layout::Layout,
    shaders::{create_composer, load_default_shaders},
};

/// The main renderer, which renders the visualization output
pub struct Renderer {
    globals: Globals,

    machine: Machine,
    atoms: Atoms,
    legend: Legend,
    time: Time,
}

impl Renderer {
    /// Creates a new [Renderer] on the passed [Device] and for the passed [TextureFormat]
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        config: &Config,
        state: &State,
        screen_resolution: (u32, u32),
    ) -> Self {
        let mut composer =
            load_default_shaders(create_composer()).expect("Failed to load default shader modules");

        let globals = Globals::new(device);

        let Layout {
            content,
            legend,
            time,
        } = Layout::new(
            screen_resolution,
            config.content_size.into(),
            36.,
            1024.,
            config.time.font.size * 1.2,
        );

        Self {
            machine: Machine::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: content,
                screen_resolution,
            }),
            atoms: Atoms::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: content,
                screen_resolution,
            }),
            legend: Legend::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: legend,
                screen_resolution,
            }),
            time: Time::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: time,
                screen_resolution,
            }),
            globals,
        }
    }

    /// Updates this [Renderer] to resemble the new [State].
    /// If `FULL` is `true`, also update this [Renderer] to resemble the new [Config].
    /// Not that all elements which depend on [State] will always update to resemble the new [State],
    /// regardless of the value of `FULL`.
    pub fn update<const FULL: bool>(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
    ) {
        self.machine
            .update::<FULL>(updater, device, queue, config, state);
        self.atoms
            .update::<FULL>(updater, device, queue, config, state);
        self.legend
            .update::<FULL>(updater, device, queue, config, state);
        self.time
            .update::<FULL>(updater, device, queue, config, state);
    }

    /// Updates the viewport resolution of this [Renderer]
    pub fn update_viewport(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_resolution: (u32, u32),
    ) {
        self.legend
            .update_viewport(device, queue, screen_resolution);
        self.machine
            .update_viewport(device, queue, screen_resolution);
        self.atoms.update_viewport(device, queue, screen_resolution);
        self.time.update_viewport(device, queue, screen_resolution);
    }

    /// Draws the contents of this [Renderer] to the passed [RenderPass]
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.rebind(render_pass);

        self.machine.draw::<true>(render_pass, self.rebind_fn());
        self.atoms.draw::<true>(render_pass, self.rebind_fn());
        self.legend.draw::<false>(render_pass, self.rebind_fn()); // No rebind: time does not need globals
        self.time.draw::<false>(render_pass, self.rebind_fn());
    }

    /// A closure which calls [Self::rebind] on `self` with the passed [RenderPass]
    #[inline]
    fn rebind_fn(&self) -> impl Fn(&mut RenderPass) + '_ {
        |r| self.rebind(r)
    }

    /// Rebinds all globals of this renderer
    #[inline]
    fn rebind(&self, render_pass: &mut RenderPass<'_>) {
        self.globals.bind(render_pass);
    }
}
