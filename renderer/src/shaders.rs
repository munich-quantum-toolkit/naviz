//! Some helper functions for linking and compiling shaders using a [Composer].

use std::{borrow::Cow, collections::HashMap};

use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor, ShaderDefValue,
};
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

/// Helper to link a shader `source` using a [Composer]
/// and then compile it on the [Device].
/// It will be given the passed `path` as a name / file-path.
/// Allows passing defines to the shader (or just [Default::default]).
pub fn compile_shader(
    device: &Device,
    composer: &mut Composer,
    source: &'static str,
    path: &'static str,
    defines: HashMap<String, ShaderDefValue>,
) -> Result<ShaderModule, Box<ComposerError>> {
    let module = composer.make_naga_module(NagaModuleDescriptor {
        source,
        file_path: path,
        shader_defs: defines,
        ..Default::default()
    })?;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        source: ShaderSource::Naga(Cow::Owned(module)),
        label: Some(path),
    });

    Ok(shader)
}

/// Creates a [Composer].
///
/// Will be validating iff compiling with debug_assertions.
pub fn create_composer() -> Composer {
    if cfg!(debug_assertions) {
        Composer::default()
    } else {
        Composer::non_validating()
    }
}

/// Loads the default shaders to the passed [Composer].
pub fn load_default_shaders(mut composer: Composer) -> Result<Composer, Box<ComposerError>> {
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("./util.wgsl"),
        file_path: "util.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("./globals.wgsl"),
        file_path: "globals.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("./viewport.wgsl"),
        file_path: "viewport.wgsl",
        ..Default::default()
    })?;

    Ok(composer)
}

#[cfg(test)]
mod test {
    use super::*;

    /// All shaders which are compiled at runtime.
    /// New shaders should be added here to have them checked.
    const SHADERS: [(&str, &str); 2] = [
        (
            include_str!("./component/primitive/lines.wgsl"),
            "lines.wgsl",
        ),
        (
            include_str!("./component/primitive/circles.wgsl"),
            "circles.wgsl",
        ),
    ];

    /// Checks that all shaders link and validate.
    ///
    /// Only covers the [Composer]-step of [compile_shader],
    /// which needs no [Device] and therefore no GPU.
    /// The [Composer] only validates iff `debug_assertions` are enabled
    /// (see [create_composer]), which is the case when running tests.
    #[test]
    fn shaders_are_valid() {
        let mut composer =
            load_default_shaders(create_composer()).expect("Failed to load default shaders");

        for (source, path) in SHADERS {
            if let Err(e) = composer.make_naga_module(NagaModuleDescriptor {
                source,
                file_path: path,
                ..Default::default()
            }) {
                panic!("Failed to compile shader {path}: {e:?}");
            }
        }
    }
}
