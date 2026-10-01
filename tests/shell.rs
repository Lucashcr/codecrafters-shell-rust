use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(3);

/// Runs the shell feeding `input` to stdin. Returns the stdout captured and the
/// exit status, or `None` for the status if the process had to be killed.
fn run_shell(input: &str) -> (String, Option<ExitStatus>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    // The shell may exit before reading everything, so ignore write errors.
    let _ = stdin.write_all(input.as_bytes());
    drop(stdin);

    // Read stdout on another thread, and cap it, since a runaway loop would
    // otherwise print prompts forever.
    let mut stdout = child.stdout.take().unwrap();
    let reader = thread::spawn(move || {
        let mut captured = Vec::new();
        let mut buf = [0u8; 4096];
        while captured.len() < 64 * 1024 {
            match stdout.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => captured.extend_from_slice(&buf[..n]),
            }
        }
        captured
    });

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

    let output = reader.join().unwrap();
    (String::from_utf8_lossy(&output).into_owned(), status)
}

fn assert_clean_run(input: &str, expected_stdout: &str) {
    let (stdout, status) = run_shell(input);
    assert_eq!(stdout, expected_stdout);
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
    assert_clean_run("foo\nexit\n", "$ foo: command not found\n$ ");
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
    assert_clean_run(
        "type\nexit\n",
        "$ type requires an argument\n$ ",
    );
}

#[test]
fn type_with_too_many_arguments() {
    assert_clean_run("type a b\nexit\n", "$ too many arguments\n$ ");
}

#[test]
fn multiple_commands_run_in_order() {
    assert_clean_run(
        "echo one\nfoo\necho two\nexit\n",
        "$ one\n$ foo: command not found\n$ two\n$ ",
    );
}

#[test]
fn blank_lines_only_reprint_prompt() {
    assert_clean_run("\n   \nexit\n", "$ \n$ \n$ ");
}

#[test]
fn nothing_runs_after_exit() {
    let (stdout, status) = run_shell("exit\necho nope\n");
    assert!(!stdout.contains("nope"), "stdout was {stdout:?}");
    assert!(status.expect("shell timed out").success());
}

// Known bug: EOF on stdin makes the shell loop forever.
#[test]
fn eof_after_commands_ends_shell() {
    assert_clean_run("echo hi\n", "$ hi\n$ \n");
}

// Known bug: EOF on stdin makes the shell loop forever.
#[test]
fn empty_stdin_ends_shell() {
    assert_clean_run("", "$ \n");
}
