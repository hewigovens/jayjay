use std::process::Command;

#[test]
fn config_prints_paste_ready_jj_configuration() {
    let output = Command::new(env!("CARGO_BIN_EXE_jayjay"))
        .arg("config")
        .output()
        .expect("run jayjay config");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 output"),
        jayjay_primitives::JJ_TOOL_CONFIG
    );
}

#[test]
fn config_rejects_extra_arguments_without_launching_the_app() {
    let output = Command::new(env!("CARGO_BIN_EXE_jayjay"))
        .args(["config", "extra"])
        .output()
        .expect("run invalid jayjay config command");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 error")
            .contains("unexpected argument 'extra'")
    );
}
