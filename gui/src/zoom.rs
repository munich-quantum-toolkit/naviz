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
            min_zoom: 0.5,
            max_zoom: 2.0,
        }
    }
}

impl ZoomState {
    /// Creates a new ZoomState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Zoom in by the specified factor, centering on the machine
    pub fn zoom_in(&mut self, factor: f32) {
        self.auto_fit = false;
        self.zoom_level = (self.zoom_level * factor).clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom out by the specified factor, centering on the machine
    pub fn zoom_out(&mut self, factor: f32) {
        self.auto_fit = false;
        self.zoom_level = (self.zoom_level / factor).clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom in towards a specific point in content coordinates
    pub fn zoom_in_towards(&mut self, factor: f32, target_point: (f32, f32)) {
        let old_zoom = self.zoom_level;
        self.zoom_in(factor);
        let new_zoom = self.zoom_level;

        // Adjust center to keep the target point stable
        let zoom_ratio = old_zoom / new_zoom;
        let dx = (target_point.0 - self.zoom_center.0) * (1.0 - zoom_ratio);
        let dy = (target_point.1 - self.zoom_center.1) * (1.0 - zoom_ratio);
        self.zoom_center.0 += dx;
        self.zoom_center.1 += dy;
    }

    /// Zoom out from a specific point in content coordinates
    pub fn zoom_out_from(&mut self, factor: f32, target_point: (f32, f32)) {
        let old_zoom = self.zoom_level;
        self.zoom_out(factor);
        let new_zoom = self.zoom_level;

        // Adjust center to keep the target point stable
        let zoom_ratio = old_zoom / new_zoom;
        let dx = (target_point.0 - self.zoom_center.0) * (1.0 - zoom_ratio);
        let dy = (target_point.1 - self.zoom_center.1) * (1.0 - zoom_ratio);
        self.zoom_center.0 += dx;
        self.zoom_center.1 += dy;
    }

    /// Set zoom to a specific level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.auto_fit = false;
        self.zoom_level = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Reset zoom to 100% and center on the machine (also enables auto-fit)
    pub fn reset_zoom(&mut self, config: &Config, atoms: &[naviz_state::state::AtomState]) {
        self.auto_fit = true; // Enable auto-fit instead of manual zoom
        self.zoom_level = 1.0;

        // Calculate the center of the machine content
        let machine_center = self.calculate_machine_center(config, atoms);
        self.zoom_center = machine_center;
    }

    /// Enable auto-fit mode (deprecated - use reset_zoom instead)
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

    /// Calculate the center point of the machine content
    pub fn calculate_machine_center(
        &self,
        config: &Config,
        atoms: &[naviz_state::state::AtomState],
    ) -> (f32, f32) {
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

        if has_content {
            ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
        } else {
            // Fall back to config extent center
            let extent = config.content_extent;
            (
                (extent.0 .0 + extent.1 .0) / 2.0,
                (extent.0 .1 + extent.1 .1) / 2.0,
            )
        }
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

        // Calculate content dimensions
        let content_width = max_x - min_x;
        let content_height = max_y - min_y;

        // Use minimal padding: 5% of content size or one grid step, whichever is smaller
        let grid_step_x = config.machine.grid.step.0;
        let grid_step_y = config.machine.grid.step.1;

        let padding_x = (content_width * 0.05).min(grid_step_x);
        let padding_y = (content_height * 0.05).min(grid_step_y);

        // Align to grid boundaries for clean appearance
        min_x = ((min_x - padding_x) / grid_step_x).floor() * grid_step_x;
        max_x = ((max_x + padding_x) / grid_step_x).ceil() * grid_step_x;
        min_y = ((min_y - padding_y) / grid_step_y).floor() * grid_step_y;
        max_y = ((max_y + padding_y) / grid_step_y).ceil() * grid_step_y;

        Some(((min_x, min_y), (max_x, max_y)))
    }

    /// Calculate the effective content extent based on zoom state and config
    pub fn calculate_effective_extent(&self, config: &Config) -> ((f32, f32), (f32, f32)) {
        if self.auto_fit {
            // Use auto-fit extent for auto mode
            if let Some(auto_extent) = self.calculate_auto_fit_extent_for_machine(config, &[]) {
                auto_extent
            } else {
                config.content_extent
            }
        } else {
            // Calculate zoomed extent without grid alignment to avoid scaling issues
            let original_extent = config.content_extent;
            let original_width = original_extent.1 .0 - original_extent.0 .0;
            let original_height = original_extent.1 .1 - original_extent.0 .1;

            // Calculate the visible area size based on zoom level
            let visible_width = original_width / self.zoom_level;
            let visible_height = original_height / self.zoom_level;

            let center_x = self.zoom_center.0;
            let center_y = self.zoom_center.1;

            let half_width = visible_width / 2.0;
            let half_height = visible_height / 2.0;

            let min_x = center_x - half_width;
            let max_x = center_x + half_width;
            let min_y = center_y - half_height;
            let max_y = center_y + half_height;

            // Don't align to grid - let the renderer handle grid scaling
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

        // Add minimal padding (5% of the dimensions)
        let width = max_x - min_x;
        let height = max_y - min_y;
        let padding_x = width * 0.05;
        let padding_y = height * 0.05;

        Some((
            (min_x - padding_x, min_y - padding_y),
            (max_x + padding_x, max_y + padding_y),
        ))
    }
}

/// UI controls for zoom functionality
pub struct ZoomControls;

impl ZoomControls {
    /// Draw zoom controls in the UI with access to config and atoms for proper centering
    pub fn draw_with_context(
        ui: &mut Ui,
        zoom_state: &mut ZoomState,
        config: Option<&Config>,
        atoms: Option<&[naviz_state::state::AtomState]>,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Zoom in button - use current center, not machine center
            if ui
                .add(Button::new("🔍+").small())
                .on_hover_text("Zoom In")
                .clicked()
            {
                // Don't change the center when manually zooming - keep current view center
                zoom_state.zoom_in(1.2);
                changed = true;
            }

            // Zoom out button - use current center, not machine center
            if ui
                .add(Button::new("🔍-").small())
                .on_hover_text("Zoom Out")
                .clicked()
            {
                // Don't change the center when manually zooming - keep current view center
                zoom_state.zoom_out(1.2);
                changed = true;
            }

            // Reset zoom button - fits to content and centers on machine
            if ui
                .add(Button::new("⌂").small())
                .on_hover_text("Fit to Content")
                .clicked()
            {
                if let (Some(config), Some(atoms)) = (config, atoms) {
                    zoom_state.reset_zoom(config, atoms);
                } else {
                    // Fallback for when context is not available
                    zoom_state.zoom_level = 1.0;
                    zoom_state.zoom_center = (0.0, 0.0);
                    zoom_state.auto_fit = true;
                }
                changed = true;
            }

            // Zoom level display (only show when not in auto-fit mode)
            if !zoom_state.auto_fit {
                ui.label(format!("{:.0}%", zoom_state.zoom_level * 100.0));
            } else {
                ui.label("Auto");
            }
        });

        changed
    }

    /// Draw zoom controls in the UI (fallback without context)
    pub fn draw(ui: &mut Ui, zoom_state: &mut ZoomState) -> bool {
        Self::draw_with_context(ui, zoom_state, None, None)
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

            // Handle left mouse button dragging for panning
            if input.pointer.primary_down() {
                let delta = input.pointer.delta();
                if delta != Vec2::ZERO {
                    // Convert screen space delta to content space delta
                    // Use the current zoom level to scale the movement properly
                    let rect = ui.max_rect();
                    let content_width = content_extent.1 .0 - content_extent.0 .0;
                    let content_height = content_extent.1 .1 - content_extent.0 .1;

                    // Calculate the scale factor from screen to content coordinates
                    let scale_x = content_width / rect.width();
                    let scale_y = content_height / rect.height();

                    // Apply the delta with proper scaling
                    let pan_x = -delta.x * scale_x;
                    let pan_y = -delta.y * scale_y;

                    zoom_state.pan((pan_x, pan_y));
                    changed = true;
                }
            }
        }

        changed
    }
}
