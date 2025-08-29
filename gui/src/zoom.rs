use egui::{Button, Ui, Vec2};
use naviz_state::config::Config;

/// Manages zoom state and operations for the visualization
#[derive(Clone, Debug)]
pub struct ZoomState {
    /// Current zoom level (1.0 = 100%, 2.0 = 200%, 0.5 = 50%)
    pub zoom_level: f32,
    /// Center point of the zoom (x, y) in content coordinates
    pub zoom_center: (f32, f32),
    /// Whether auto-fit is enabled
    pub auto_fit: bool,
    /// Minimum allowed zoom level
    pub min_zoom: f32,
    /// Maximum allowed zoom level
    pub max_zoom: f32,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            zoom_center: (0.0, 0.0),
            auto_fit: true,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

impl ZoomState {
    /// Creates a new ZoomState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Zoom in by the specified factor
    pub fn zoom_in(&mut self, factor: f32) {
        self.auto_fit = false;
        self.zoom_level = (self.zoom_level * factor).clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom out by the specified factor
    pub fn zoom_out(&mut self, factor: f32) {
        self.auto_fit = false;
        self.zoom_level = (self.zoom_level / factor).clamp(self.min_zoom, self.max_zoom);
    }

    /// Set zoom to a specific level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.auto_fit = false;
        self.zoom_level = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self) {
        self.auto_fit = false;
        self.zoom_level = 1.0;
        self.zoom_center = (0.0, 0.0);
    }

    /// Enable auto-fit mode
    pub fn enable_auto_fit(&mut self) {
        self.auto_fit = true;
    }

    /// Pan the view by the specified offset in content coordinates
    pub fn pan(&mut self, offset: (f32, f32)) {
        self.auto_fit = false;
        self.zoom_center.0 += offset.0;
        self.zoom_center.1 += offset.1;
    }

    /// Set the zoom center to a specific point
    pub fn set_center(&mut self, center: (f32, f32)) {
        self.auto_fit = false;
        self.zoom_center = center;
    }

    /// Calculate auto-fit extent based on the entire machine layout, not just atoms
    pub fn calculate_auto_fit_extent_for_machine(
        &self,
        config: &Config,
        atoms: &[naviz_state::state::AtomState],
    ) -> Option<((f32, f32), (f32, f32))> {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        let mut has_content = false;

        // Include atom positions
        for atom in atoms {
            let (x, y) = atom.position;
            let radius = atom.size;

            min_x = min_x.min(x - radius);
            max_x = max_x.max(x + radius);
            min_y = min_y.min(y - radius);
            max_y = max_y.max(y + radius);
            has_content = true;
        }

        // Include trap positions
        for &trap_pos in &config.machine.traps.positions {
            let radius = config.machine.traps.radius;
            min_x = min_x.min(trap_pos.0 - radius);
            max_x = max_x.max(trap_pos.0 + radius);
            min_y = min_y.min(trap_pos.1 - radius);
            max_y = max_y.max(trap_pos.1 + radius);
            has_content = true;
        }

        // Include zone boundaries
        for zone in &config.machine.zones {
            let zone_min_x = zone.start.0;
            let zone_max_x = zone.start.0 + zone.size.0;
            let zone_min_y = zone.start.1;
            let zone_max_y = zone.start.1 + zone.size.1;

            min_x = min_x.min(zone_min_x);
            max_x = max_x.max(zone_max_x);
            min_y = min_y.min(zone_min_y);
            max_y = max_y.max(zone_max_y);
            has_content = true;
        }

        // If no content found, fall back to original content extent
        if !has_content {
            return Some(config.content_extent);
        }

        // Ensure we include at least some grid lines for context
        let grid_step_x = config.machine.grid.step.0;
        let grid_step_y = config.machine.grid.step.1;

        // Align to grid boundaries and add some grid padding
        let grid_padding_x = grid_step_x * 2.0;
        let grid_padding_y = grid_step_y * 2.0;

        // Align boundaries to grid steps
        min_x = (min_x / grid_step_x).floor() * grid_step_x - grid_padding_x;
        max_x = (max_x / grid_step_x).ceil() * grid_step_x + grid_padding_x;
        min_y = (min_y / grid_step_y).floor() * grid_step_y - grid_padding_y;
        max_y = (max_y / grid_step_y).ceil() * grid_step_y + grid_padding_y;

        Some(((min_x, min_y), (max_x, max_y)))
    }

    /// Calculate the effective content extent based on zoom state and config
    pub fn calculate_effective_extent(&self, config: &Config) -> ((f32, f32), (f32, f32)) {
        let original_extent = config.content_extent;

        if self.auto_fit {
            // Return the original extent for auto-fit mode
            original_extent
        } else {
            // Calculate zoomed extent aligned to grid
            let grid_step_x = config.machine.grid.step.0;
            let grid_step_y = config.machine.grid.step.1;

            let original_width = original_extent.1 .0 - original_extent.0 .0;
            let original_height = original_extent.1 .1 - original_extent.0 .1;

            let zoomed_width = original_width / self.zoom_level;
            let zoomed_height = original_height / self.zoom_level;

            let center_x = self.zoom_center.0;
            let center_y = self.zoom_center.1;

            let half_width = zoomed_width / 2.0;
            let half_height = zoomed_height / 2.0;

            let mut min_x = center_x - half_width;
            let mut max_x = center_x + half_width;
            let mut min_y = center_y - half_height;
            let mut max_y = center_y + half_height;

            // Align to grid boundaries to maintain coordinate system alignment
            min_x = (min_x / grid_step_x).floor() * grid_step_x;
            max_x = (max_x / grid_step_x).ceil() * grid_step_x;
            min_y = (min_y / grid_step_y).floor() * grid_step_y;
            max_y = (max_y / grid_step_y).ceil() * grid_step_y;

            ((min_x, min_y), (max_x, max_y))
        }
    }

    /// Calculate auto-fit extent based on actual atom positions
    pub fn calculate_auto_fit_extent(
        &self,
        atoms: &[naviz_state::state::AtomState],
    ) -> Option<((f32, f32), (f32, f32))> {
        if atoms.is_empty() {
            return None;
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for atom in atoms {
            let (x, y) = atom.position;
            let radius = atom.size;

            min_x = min_x.min(x - radius);
            max_x = max_x.max(x + radius);
            min_y = min_y.min(y - radius);
            max_y = max_y.max(y + radius);
        }

        // Add some padding (10% of the dimensions)
        let width = max_x - min_x;
        let height = max_y - min_y;
        let padding_x = width * 0.1;
        let padding_y = height * 0.1;

        Some((
            (min_x - padding_x, min_y - padding_y),
            (max_x + padding_x, max_y + padding_y),
        ))
    }
}

/// UI controls for zoom functionality
pub struct ZoomControls;

impl ZoomControls {
    /// Draw zoom controls in the UI
    pub fn draw(ui: &mut Ui, zoom_state: &mut ZoomState) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Zoom in button
            if ui
                .add(Button::new("🔍+").small())
                .on_hover_text("Zoom In")
                .clicked()
            {
                zoom_state.zoom_in(1.2);
                changed = true;
            }

            // Zoom out button
            if ui
                .add(Button::new("🔍-").small())
                .on_hover_text("Zoom Out")
                .clicked()
            {
                zoom_state.zoom_out(1.2);
                changed = true;
            }

            // Reset zoom button
            if ui
                .add(Button::new("⌂").small())
                .on_hover_text("Reset Zoom")
                .clicked()
            {
                zoom_state.reset_zoom();
                changed = true;
            }

            // Fit to content button
            if ui
                .add(Button::new("⤢").small())
                .on_hover_text("Fit to Content")
                .clicked()
            {
                zoom_state.enable_auto_fit();
                changed = true;
            }

            // Zoom level display
            ui.label(format!("{:.0}%", zoom_state.zoom_level * 100.0));

            // Auto-fit indicator
            if zoom_state.auto_fit {
                ui.label("📐").on_hover_text("Auto-fit enabled");
            }
        });

        changed
    }

    /// Handle mouse interactions for zoom and pan
    pub fn handle_mouse_interaction(
        ui: &mut Ui,
        zoom_state: &mut ZoomState,
        content_extent: ((f32, f32), (f32, f32)),
    ) -> bool {
        let mut changed = false;

        if ui.rect_contains_pointer(ui.max_rect()) {
            let input = ui.input(|i| i.clone());

            // Handle scroll wheel for zooming
            let scroll_delta = input.smooth_scroll_delta;
            if scroll_delta.y != 0.0 {
                let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 1.0 / 1.1 };

                // Get mouse position relative to the content
                if let Some(pointer_pos) = input.pointer.hover_pos() {
                    let rect = ui.max_rect();
                    let relative_pos = Vec2::new(
                        (pointer_pos.x - rect.left()) / rect.width(),
                        (pointer_pos.y - rect.top()) / rect.height(),
                    );

                    // Convert to content coordinates
                    let content_width = content_extent.1 .0 - content_extent.0 .0;
                    let content_height = content_extent.1 .1 - content_extent.0 .1;
                    let content_pos = (
                        content_extent.0 .0 + relative_pos.x * content_width,
                        content_extent.0 .1 + relative_pos.y * content_height,
                    );

                    // Zoom towards the mouse position
                    let old_zoom = zoom_state.zoom_level;
                    if scroll_delta.y > 0.0 {
                        zoom_state.zoom_in(zoom_factor);
                    } else {
                        zoom_state.zoom_out(zoom_factor);
                    }
                    let new_zoom = zoom_state.zoom_level;

                    // Adjust center to keep the mouse position stable
                    let zoom_ratio = old_zoom / new_zoom;
                    let dx = (content_pos.0 - zoom_state.zoom_center.0) * (1.0 - zoom_ratio);
                    let dy = (content_pos.1 - zoom_state.zoom_center.1) * (1.0 - zoom_ratio);
                    zoom_state.zoom_center.0 += dx;
                    zoom_state.zoom_center.1 += dy;

                    changed = true;
                }
            }

            // Handle middle mouse button dragging for panning
            if input.pointer.middle_down() {
                let delta = input.pointer.delta();
                if delta != Vec2::ZERO {
                    let rect = ui.max_rect();
                    let content_width = content_extent.1 .0 - content_extent.0 .0;
                    let content_height = content_extent.1 .1 - content_extent.0 .1;

                    let pan_x = -(delta.x / rect.width()) * content_width / zoom_state.zoom_level;
                    let pan_y = -(delta.y / rect.height()) * content_height / zoom_state.zoom_level;

                    zoom_state.pan((pan_x, pan_y));
                    changed = true;
                }
            }
        }

        changed
    }
}
