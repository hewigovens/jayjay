use std::panic::{AssertUnwindSafe, catch_unwind};

use jayjay_primitives::CliCommandOutcome;

// A panic must not cross the FFI: this export is infallible, so the generated Swift would fatalError with no diagnostic; map it to an error outcome like the shells print for any other failure.
#[uniffi::export]
fn run_app_cli_command(arguments: Vec<String>, version: String) -> Option<CliCommandOutcome> {
    catch_unwind(AssertUnwindSafe(|| {
        jayjay_core::run_app_cli_command(&arguments, &version)
    }))
    .unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .map(str::to_owned)
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "internal error".to_owned());
        Some(CliCommandOutcome::err(format!("error: {message}\n")))
    })
}
