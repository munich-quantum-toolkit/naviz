use naviz_state::{config::Config, state::State};
use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    buffer_updater::BufferUpdater,
    component::{
        atoms::Atoms,
        drawable::{Drawable, Hidable},
        legend::Legend,
        machine::Machine,
        time::Time,
        updatable::Updatable,
        ComponentInit,
    },
    globals::Globals,
    layout::Layout,
    shaders::{create_composer, load_default_shaders},
    viewport::{ViewportProjection, ViewportSource},
};

/// The main renderer, which renders the visualization output
pub struct Renderer {
    globals: Globals,

    machine: Machine,
    atoms: Atoms,
    legend: Hidable<Legend>,
    time: Hidable<Time>,
    screen_resolution: (u32, u32),
    /// Whether to force the [content-only-layout][Layout::new_content_only].
    /// Independent of the selected style.
    force_zen: bool,
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
        } = get_layout(config, screen_resolution, false);

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
            legend: Hidable::new(Legend::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: legend.unwrap_or(ViewportProjection::identity()),
                screen_resolution,
            }))
            .with_visibility(legend.is_some()),
            time: Hidable::new(Time::new(ComponentInit {
                device,
                queue,
                format,
                globals: &globals,
                shader_composer: &mut composer,
                config,
                state,
                viewport_projection: time.unwrap_or(ViewportProjection::identity()),
                screen_resolution,
            }))
            .with_visibility(time.is_some()),
            globals,
            screen_resolution,
            force_zen: false,
        }
    }

    /// Whether to force the [content-only-layout][Layout::new_content_only].
    /// Independent of the selected style.
    pub fn set_force_zen(&mut self, force_zen: bool) {
        self.force_zen = force_zen;
    }

    /// Updates this [Renderer] to resemble the new [State].
    /// See [Updatable::update].
    pub fn update(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
    ) {
        self.machine.update(updater, device, queue, config, state);
        self.atoms.update(updater, device, queue, config, state);
        self.legend.update(updater, device, queue, config, state);
        self.time.update(updater, device, queue, config, state);
    }

    /// Updates this [Renderer] to resemble the new [State] and [Config].
    /// See [Updatable::update_full].
    pub fn update_full(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
    ) {
        let Layout {
            content,
            legend,
            time,
        } = get_layout(config, self.screen_resolution, self.force_zen);

        self.machine
            .update_full(updater, device, queue, config, state, content);
        self.atoms
            .update_full(updater, device, queue, config, state, content);
        self.legend.update_full(
            updater,
            device,
            queue,
            config,
            state,
            legend.unwrap_or(ViewportProjection::identity()),
        );
        self.legend.set_visible(legend.is_some());
        self.time.update_full(
            updater,
            device,
            queue,
            config,
            state,
            time.unwrap_or(ViewportProjection::identity()),
        );
        self.time.set_visible(time.is_some());
    }

    /// Updates the viewport resolution of this [Renderer]
    pub fn update_viewport(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_resolution: (u32, u32),
    ) {
        self.screen_resolution = screen_resolution;
        self.legend
            .update_viewport(device, queue, screen_resolution);
        self.machine
            .update_viewport(device, queue, screen_resolution);
        self.atoms.update_viewport(device, queue, screen_resolution);
        self.time.update_viewport(device, queue, screen_resolution);
    }

    /// Draws the contents of this [Renderer] to the passed [RenderPass]
    pub fn draw(&self, render_pass: &mut RenderPass<'_>) {
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

/// Gets the [Layout] to use based on the passed [Config].
/// Will detect which [Layout] to use based on which parts should be displayed in the [Config].
/// If `force_content_only` is `true`, will always use [Layout::new_content_only].
fn get_layout(config: &Config, screen_resolution: (u32, u32), force_content_only: bool) -> Layout {
    const LEGEND_HEIGHT: f32 = 1024.;

    // content source
    let content = ViewportSource::from_tl_br(config.content_extent.0, config.content_extent.1);

    // Calculate dynamic content padding based on grid legend configuration and content dimensions
    let content_padding_y =
        calculate_content_padding(&config.machine.grid.legend, content.width, content.height);

    if force_content_only || (!config.display_time() && !config.display_sidebar()) {
        // no time and no sidebar
        Layout::new_content_only(screen_resolution, content, content_padding_y)
    } else {
        // default layout
        Layout::new_full(
            screen_resolution,
            content,
            content_padding_y,
            LEGEND_HEIGHT,
            config.time.font.size * 1.2,
        )
    }
}

/// Calculates appropriate content padding based on the grid legend configuration.
/// This replaces the hard-coded padding with dynamic calculation that considers:
/// - Font size of coordinate labels
/// - Whether labels and numbers are displayed
/// - Additional space for text descenders and ascenders
/// - Space for axis label identifiers (x, y labels)
/// - Content aspect ratio to avoid excessive padding on tall/wide machines
fn calculate_content_padding(
    grid_legend: &naviz_state::config::GridLegendConfig,
    content_width: f32,
    content_height: f32,
) -> f32 {
    const MIN_CONTENT_PADDING: f32 = 12.0; // Minimum padding for visual breathing room
    const FONT_SIZE_MULTIPLIER: f32 = 2.5; // Multiplier to account for ascenders/descenders and spacing
    const AXIS_LABEL_EXTRA: f32 = 15.0; // Extra space for axis label identifiers (x, y)
    const MAX_ASPECT_RATIO_SCALING: f32 = 4.0; // Cap the aspect ratio scaling effect

    // If neither labels nor numbers are displayed, use minimal padding
    if !grid_legend.display_labels && !grid_legend.display_numbers {
        return MIN_CONTENT_PADDING;
    }

    // Calculate base padding from font size
    let mut padding = grid_legend.font.size * FONT_SIZE_MULTIPLIER;

    // Add extra space if labels are displayed (for axis identifiers like "x", "y")
    if grid_legend.display_labels {
        padding += AXIS_LABEL_EXTRA;
    }

    // Apply aggressive scaling for non-square machines
    let aspect_ratio = content_height / content_width;

    if aspect_ratio > 1.2 {
        // For tall machines: use quadratic reduction that gets more aggressive as machines get taller
        let clamped_ratio = aspect_ratio.min(MAX_ASPECT_RATIO_SCALING);
        let mut reduction_factor = 1.2 / clamped_ratio;
        reduction_factor = reduction_factor * reduction_factor;
        padding *= reduction_factor;
    } else if aspect_ratio < 0.83 {
        // For wide machines: similar aggressive scaling
        let clamped_ratio = aspect_ratio.max(1.0 / MAX_ASPECT_RATIO_SCALING);
        let mut reduction_factor = clamped_ratio / 0.83;
        reduction_factor = reduction_factor * reduction_factor;
        padding *= reduction_factor;
    }

    // Use the larger of minimum padding or calculated padding
    padding.max(MIN_CONTENT_PADDING)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_layout_has_all_sections() {
        let config = Config::example();

        let layout = get_layout(&config, (1920, 1080), false);

        assert!(
            layout.legend.is_some(),
            "Example config should produce a layout with space for the legend"
        );
        assert!(
            layout.time.is_some(),
            "Example config should produce a layout with space for the time"
        );
    }

    #[test]
    fn example_layout_display_none_only_content() {
        let mut config = Config::example();
        config.legend.entries = Vec::new(); // No legend
        config.time.display = false; // No time

        let layout = get_layout(&config, (1920, 1080), false);

        assert!(
            layout.legend.is_none(),
            "Example config without legend and time should not allocates space for legend"
        );
        assert!(
            layout.time.is_none(),
            "Example config without legend and time should not allocates space for time"
        );
    }

    #[test]
    fn calculate_content_padding_with_labels_and_numbers() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 12.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For content (800x600): aspect ratio = 0.75 < 0.83, so wide machine scaling applies
        // reduction_factor = 0.75 / 0.83 = 0.903, squared = 0.816
        // padding = 45.0 * 0.816 ≈ 36.74
        let padding = calculate_content_padding(&grid_legend, 800.0, 600.0);
        assert!(
            (padding - 36.74).abs() < 0.01,
            "Expected ~36.74, got {}",
            padding
        );
    }

    #[test]
    fn calculate_content_padding_no_display() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 12.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: false,
            display_numbers: false,
        };

        let padding = calculate_content_padding(&grid_legend, 800.0, 600.0);
        assert_eq!(padding, 12.0); // MIN_CONTENT_PADDING
    }

    #[test]
    fn calculate_content_padding_large_font() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 24.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For content (800x600): aspect ratio = 0.75 < 0.83, so wide machine scaling applies
        // base padding = 24.0 * 2.5 + 15.0 = 75.0
        // reduction_factor = 0.75 / 0.83 = 0.903, squared = 0.816
        // padding = 75.0 * 0.816 ≈ 61.24
        let padding = calculate_content_padding(&grid_legend, 800.0, 600.0);
        assert!(
            (padding - 61.24).abs() < 0.01,
            "Expected ~61.24, got {}",
            padding
        );
    }

    #[test]
    fn calculate_content_padding_small_font() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 4.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For content (800x600): aspect ratio = 0.75 < 0.83, so wide machine scaling applies
        // base padding = 4.0 * 2.5 + 15.0 = 25.0
        // reduction_factor = 0.75 / 0.83 = 0.903, squared = 0.816
        // padding = 25.0 * 0.816 ≈ 20.41
        let padding = calculate_content_padding(&grid_legend, 800.0, 600.0);
        assert!(
            (padding - 20.41).abs() < 0.01,
            "Expected ~20.41, got {}",
            padding
        );
    }

    #[test]
    fn calculate_content_padding_tall_machine() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 12.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For very tall machine (100x2000): aspect ratio = 20.0 > 1.2, clamped to 4.0
        // reduction_factor = 1.2 / 4.0 = 0.3, squared = 0.09
        // padding = 45.0 * 0.09 = 4.05, but max with MIN_CONTENT_PADDING = 12.0
        let padding = calculate_content_padding(&grid_legend, 100.0, 2000.0);
        assert_eq!(padding, 12.0); // Should be the minimum padding
    }

    #[test]
    fn calculate_content_padding_square_machine() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 12.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For square machine (500x500): aspect ratio = 1.0, between 0.67 and 1.5, so no scaling
        let padding = calculate_content_padding(&grid_legend, 500.0, 500.0);
        assert_eq!(padding, 45.0);
    }

    #[test]
    fn calculate_content_padding_moderately_tall_machine() {
        use naviz_state::config::{FontConfig, GridLegendConfig, HPosition, VPosition};

        let grid_legend = GridLegendConfig {
            step: (40., 40.),
            font: FontConfig {
                size: 12.,
                color: [16, 16, 16, 255],
                family: "Fira Mono".to_owned(),
            },
            labels: ("x".to_owned(), "y".to_owned()),
            position: (VPosition::Bottom, HPosition::Left),
            display_labels: true,
            display_numbers: true,
        };

        // For moderately tall machine (100x300): aspect ratio = 3.0 > 1.2
        // reduction_factor = 1.2 / 3.0 = 0.4, squared = 0.16
        // padding = 45.0 * 0.16 = 7.2, but max with MIN_CONTENT_PADDING = 12.0
        let padding = calculate_content_padding(&grid_legend, 100.0, 300.0);
        assert_eq!(padding, 12.0);
    }
}
