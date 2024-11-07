use naviz_state::{config::Config, state::State};
use wgpu::{Device, Queue};

use crate::{buffer_updater::BufferUpdater, viewport::ViewportProjection};

/// A trait for components that can be updated.
///
/// This allows specifying whether only the state has changed ([Updatable::update])
/// or if the config changed as well ([Updatable::update_full]).
pub trait Updatable {
    /// Updates this [Updatable] to resemble the new [State].
    /// The passed [Config] is assumed to be unchanged
    /// (i.e., components that only depend on [Config] will not be updated,
    /// however components that _do_ depend on [State] _will_ use the updated [Config]).
    /// If the [Config] changed, use [Updatable::update_full] instead.
    fn update(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
    );

    /// Updates this [Updatable] to resemble the new [State], [Config], and [ViewportProjection].
    /// For updates, which _only_ changed the [State], use [Updatable::update] instead.
    fn update_full(
        &mut self,
        updater: &mut impl BufferUpdater,
        device: &Device,
        queue: &Queue,
        config: &Config,
        state: &State,
        viewport_projection: ViewportProjection,
    );
}
