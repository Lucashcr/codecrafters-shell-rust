use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(3);

/// Runs the shell feeding `input` to stdin. Returns the stdout captured and the
/// exit status, or `None` for the status if the process had to be killed.
fn run_shell(input: &str) -> (String, Option<ExitStatus>) {
    run_shell_with_path(input, None)
}

/// Like `run_shell`, but overrides the `PATH` the shell indexes at startup.
fn run_shell_with_path(input: &str, path: Option<&str>) -> (String, Option<ExitStatus>) {
    let (stdout, _stderr, status) = run_shell_capturing_all(input, path);
    (stdout, status)
}

/// Reads `source` on another thread, capped, since a runaway loop would
/// otherwise print forever.
fn capture(mut source: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut buf = [0u8; 4096];
        while captured.len() < 64 * 1024 {
            match source.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => captured.extend_from_slice(&buf[..n]),
            }
        }
        captured
    })
}

/// Like `run_shell_with_path`, but also returns the captured stderr.
fn run_shell_capturing_all(
    input: &str,
    path: Option<&str>,
) -> (String, String, Option<ExitStatus>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"));
    if let Some(path) = path {
        command.env("PATH", path);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    // The shell may exit before reading everything, so ignore write errors.
    let _ = stdin.write_all(input.as_bytes());
    drop(stdin);

    let stdout_reader = capture(child.stdout.take().unwrap());
    let stderr_reader = capture(child.stderr.take().unwrap());

    let deadline = Instant::now() + TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = stdout_reader.join().unwrap();
    let stderr = stderr_reader.join().unwrap();
    (
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
        status,
    )
}

fn assert_clean_run(input: &str, expected_stdout: &str) {
    assert_clean_run_with_stderr(input, expected_stdout, "");
}

fn assert_clean_run_with_stderr(input: &str, expected_stdout: &str, expected_stderr: &str) {
    let (stdout, stderr, status) = run_shell_capturing_all(input, None);
    assert_eq!(stdout, expected_stdout);
    assert_eq!(stderr, expected_stderr);
    assert!(status.expect("shell timed out").success());
}

#[test]
fn exit_ends_shell() {
    assert_clean_run("exit\n", "$ ");
}

#[test]
fn echo_prints_arguments() {
    assert_clean_run("echo hello world\nexit\n", "$ hello world\n$ ");
}

#[test]
fn unknown_command_is_reported() {
    assert_clean_run_with_stderr("foo\nexit\n", "$ $ ", "foo: command not found\n");
}

#[test]
fn type_builtins() {
    assert_clean_run("type echo\nexit\n", "$ echo is a shell builtin\n$ ");
    assert_clean_run("type exit\nexit\n", "$ exit is a shell builtin\n$ ");
    assert_clean_run("type type\nexit\n", "$ type is a shell builtin\n$ ");
}

#[test]
fn type_unknown() {
    assert_clean_run("type foo\nexit\n", "$ foo: not found\n$ ");
}

#[test]
fn type_without_argument() {
    assert_clean_run("type\nexit\n", "$ type requires an argument\n$ ");
}

#[test]
fn type_with_too_many_arguments() {
    assert_clean_run("type a b\nexit\n", "$ too many arguments\n$ ");
}

#[test]
fn multiple_commands_run_in_order() {
    assert_clean_run_with_stderr(
        "echo one\nfoo\necho two\nexit\n",
        "$ one\n$ $ two\n$ ",
        "foo: command not found\n",
    );
}

#[test]
fn blank_lines_only_reprint_prompt() {
    assert_clean_run("\n   \nexit\n", "$ \n$ \n$ ");
}

#[test]
fn missing_path_directory_prints_nothing_extra() {
    let (stdout, status) = run_shell_with_path("exit\n", Some("/nonexistent/dir"));
    assert_eq!(stdout, "$ ");
    assert!(status.expect("shell timed out").success());
}

#[test]
fn nothing_runs_after_exit() {
    let (stdout, status) = run_shell("exit\necho nope\n");
    assert!(!stdout.contains("nope"), "stdout was {stdout:?}");
    assert!(status.expect("shell timed out").success());
}

#[test]
fn eof_after_commands_ends_shell() {
    assert_clean_run("echo hi\n", "$ hi\n$ \n");
}

#[test]
fn empty_stdin_ends_shell() {
    assert_clean_run("", "$ \n");
}
