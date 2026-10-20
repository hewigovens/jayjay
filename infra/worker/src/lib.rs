#[cfg(any(target_arch = "wasm32", test))]
mod telemetry;

#[cfg(target_arch = "wasm32")]
mod handler;
