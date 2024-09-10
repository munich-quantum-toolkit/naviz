/// Whether the app runs on a web-platform
pub const WEB: bool = cfg!(target_arch = "wasm32");
