extern crate assert_cmd;
extern crate predicates;
extern crate tempfile;

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

static SAMPLE_FILE: &str = r#"
We need some test data to search for stuff, so here goes...

somestring
 somestring
someotherstring
SoMEsTriNGWIthObNoXiouSCaps
SOMESTRINGINALLCAPS
repeat repeated repeated
repeat repeated

Unicode is fun! 🦀
Hello, 世界!
"#;

#[test]
fn no_args() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .assert()
        .code(2)
        .stderr(predicate::str::is_empty().not())
        .stdout(predicate::str::is_empty());
}

#[test]
fn invalid_flag() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["--does-not-exist", "validpattern"])
        .assert()
        .code(2)
        .stderr(predicate::str::is_empty().not())
        .stdout(predicate::str::is_empty());
}

#[test]
fn help() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .arg("--help")
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("OPTIONS"));
}

#[test]
fn empty_input() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["somepattern", "/dev/null"])
        .assert()
        .code(1)
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());
}

#[test]
fn simple_match() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .arg("foo")
        .write_stdin("foo\n")
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("foo\n"));
}

#[test]
fn count_empty_input() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-c", "somepattern", "/dev/null"])
        .assert()
        .code(1)
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("0\n"));
}

#[test]
fn simple_count_from_sample() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-c", "repeat"])
        .write_stdin(SAMPLE_FILE)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("2\n"));
}

#[test]
fn unicode_match_from_sample() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["🦀"])
        .write_stdin(SAMPLE_FILE)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("Unicode is fun! 🦀\n"));
}

#[test]
fn simple_match_from_file() {
    let mut file = NamedTempFile::new().expect("temp file");
    write!(file, "{}", SAMPLE_FILE).expect("wrote temp file");
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["somestring", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("somestring\n somestring\n"));
}

#[test]
fn single_file_with_headers() {
    let mut file = NamedTempFile::new().expect("temp file");
    write!(file, "{}", SAMPLE_FILE).expect("wrote temp file");
    let filename = file.path().to_str().unwrap();
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-H", "someother", &filename])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::starts_with(format!("{}:", &filename)));
}

#[test]
fn multiple_files() {
    let mut file1 = NamedTempFile::new().expect("temp file");
    write!(file1, "{}", "nothing interesting").expect("wrote temp file");
    let filename1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().expect("temp file");
    write!(file2, "{}", SAMPLE_FILE).expect("wrote temp file");
    let filename2 = file2.path().to_str().unwrap();

    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["someother", &filename1, &filename2])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::starts_with(format!("{}:", &filename2)));
}

#[test]
fn multiple_files_without_headers() {
    let mut file1 = NamedTempFile::new().expect("temp file");
    write!(file1, "{}", "nothing interesting").expect("wrote temp file");
    let filename1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().expect("temp file");
    write!(file2, "{}", SAMPLE_FILE).expect("wrote temp file");
    let filename2 = file2.path().to_str().unwrap();

    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-h", "someother", &filename1, &filename2])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("someotherstring\n"));
}

#[test]
fn line_numbers() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-n", "Hello"])
        .write_stdin(SAMPLE_FILE)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::starts_with("13:Hello"));
}

#[test]
fn stdin_header_and_line_numbers() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-H", "-n", "Hello"])
        .write_stdin(SAMPLE_FILE)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::starts_with("(standard input):13:Hello"));
}

// If there's an error on a file, that should be reflected on stderr and
// in the return code, but we should still look at any remaining files.
#[test]
fn error_on_first_of_multiple_files() {
    let mut file = NamedTempFile::new().expect("temp file");
    write!(file, "{}", SAMPLE_FILE).expect("wrote temp file");
    let filename = file.path().to_str().unwrap();
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["someother", "nonexistentfilefirst", &filename])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("No such file or directory"))
        .stdout(predicate::str::contains("someotherstring"));
}

#[test]
fn headers_on_counts() {
    let mut file1 = NamedTempFile::new().expect("temp file");
    write!(file1, "{}", "garbage\n").expect("wrote temp file");
    let filename1 = file1.path().to_str().unwrap();

    let mut file2 = NamedTempFile::new().expect("temp file");
    write!(file2, "{}", SAMPLE_FILE).expect("wrote temp file");
    let filename2 = file2.path().to_str().unwrap();

    let expected = format!("{}:0\n{}:2\n", &filename1, &filename2);
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-c", "somestring", &filename1, &filename2])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar(expected));
}

#[test]
fn case_insensitive_match() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-i", "foo"])
        .write_stdin("Foo\n")
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("Foo\n"));
}

#[test]
fn quiet_match() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-q", "foo"])
        .write_stdin("foo\n")
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());
}

#[test]
fn max_count() {
    Command::cargo_bin("grep")
        .expect("found binary")
        .args(&["-cm2", "some"])
        .write_stdin(SAMPLE_FILE)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::similar("2\n"));
}
