use crate::flags;
use anyhow::Context;
use xshell::{cmd, Shell};

pub fn run(sh: &Shell, flags: flags::Pluginharness) -> anyhow::Result<()> {
    let _pd = sh.push_dir(crate::project_root());
    let cargo = crate::cargo()?;
    let err_context = "failed to run plugin harness checks";

    crate::status(">> Plugin Harness");
    println!("\n>> Plugin harness: deterministic plugin-platform checks");

    let run_test = |test_filter: &str| -> anyhow::Result<()> {
        cmd!(
            sh,
            "{cargo} test -p zellij-utils {test_filter} -- --nocapture"
        )
        .env("RUST_TEST_THREADS", "1")
        .env("TZ", "UTC")
        .env("ZELLIJ_PLUGIN_HARNESS", "1")
        .run()
        .with_context(|| format!("failed running plugin harness test filter '{test_filter}'"))
    };

    run_test("permission_cache_matches")?;
    run_test("get_granted_plugin_permissions_command")?;
    run_test("request_plugin_state_snapshot_command")?;
    run_test("get_plugin_logs_command")?;
    run_test("pipe_message_envelope_v2_roundtrip")?;

    if !flags.skip_check {
        cmd!(sh, "{cargo} check -p zellij-server -p zellij-tile")
            .env("RUST_TEST_THREADS", "1")
            .env("TZ", "UTC")
            .env("ZELLIJ_PLUGIN_HARNESS", "1")
            .run()
            .context(err_context)?;
    }

    Ok(())
}
