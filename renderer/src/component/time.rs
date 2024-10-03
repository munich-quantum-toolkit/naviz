use naviz_state::{config::Config, state::State};
use wgpu::{Device, Queue, RenderPass};

use crate::{buffer_updater::BufferUpdater, viewport::ViewportProjection};

use super::{
    primitive::text::{Alignment, HAlignment, Text, TextSpec, VAlignment},
    ComponentInit,
};

/// A component to display the time on the screen
pub struct Time {
    text: Text,
    viewport_projection: ViewportProjection,
}

impl Time {
    pub fn new(
        ComponentInit {
            device,
            queue,
            format,
            globals: _,
            shader_composer: _,
            config,
            state,
            viewport_projection,
            screen_resolution,
        }: ComponentInit,
    ) -> Self {
        Self {
            text: Text::new(
                device,
                queue,
                format,
                get_specs(config, state, viewport_projection),
                screen_resolution,
            ),
            viewport_projection,
        }
    }

    /// Updates this [Time] to resemble the new [State].
    /// If `FULL` is `true`, also update this [Time] to resemble the new [Config].
    /// Not that all elements which depend on [State] will always update to resemble the new [State],
    /// regardless of the value of `FULL`.
    pub fn update<const FULL: bool>(
        &mut self,
        _updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
    ) {
        self.text.update(
            (device, queue),
            get_specs(config, state, self.viewport_projection),
        );
    }

    /// Updates the viewport resolution of this [Time]
    pub fn update_viewport(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_resolution: (u32, u32),
    ) {
        self.text
            .update_viewport((device, queue), screen_resolution);
    }

    /// Draws this [Time].
    ///
    /// May overwrite bind groups.
    /// If `REBIND` is `true`, will call the passed `rebind`-function to rebind groups.
    #[inline]
    pub fn draw<'a, const REBIND: bool>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        rebind: impl Fn(&mut RenderPass),
    ) {
        self.text.draw::<REBIND>(render_pass, rebind);
    }
}

/// Gets the specs for [Time] from the passed [State] and [Config].
fn get_specs<'a>(
    config: &'a Config,
    state: &'a State,
    viewport_projection: ViewportProjection,
) -> TextSpec<'a, impl IntoIterator<Item = (&'a str, (f32, f32), Alignment)>> {
    TextSpec {
        viewport_projection,
        font_size: config.time.font.size,
        font_family: &config.time.font.family,
        texts: [(
            state.time.as_str(),
            (0., viewport_projection.source.height / 2.),
            Alignment(HAlignment::Left, VAlignment::Center),
        )],
        color: config.time.font.color,
    }
}
