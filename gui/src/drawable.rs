use egui::Ui;

/// Something that can be drawn to a [Ui].
pub trait Drawable {
    /// Draws this [Drawable] to the [Ui]
    fn draw(self, ui: &mut Ui);
}
