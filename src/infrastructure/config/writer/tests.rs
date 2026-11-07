#![cfg(test)]

use std::time::Duration;

use super::*;

#[test]
fn merges_quick_saves_into_one_write_with_the_latest_text() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("config.json");
    let writer = Writer::new();
    assert!(writer.spawn());
    for index in 0..20 {
        writer.enqueue(&path, format!("{{\"n\": {index}}}").into_bytes());
    }
    assert!(writer.is_pending(&path));
    assert!(writer.flush(Duration::from_secs(5)));
    assert!(!writer.is_pending(&path));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "{\"n\": 19}");
    assert_eq!(writer.written(), 1);
}

#[test]
fn writes_by_itself_after_the_debounce() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("config.json");
    let writer = Writer::new();
    assert!(writer.spawn());
    writer.enqueue(&path, b"one".to_vec());
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while writer.written() == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "one");
}

#[test]
fn flush_without_pending_writes_returns_at_once() {
    let writer = Writer::new();
    assert!(writer.flush(Duration::from_millis(1)));
}

#[test]
fn keeps_separate_files_apart() {
    let dir = tempfile::tempdir().expect("temp dir");
    let first = dir.path().join("a.json");
    let second = dir.path().join("b.json");
    let writer = Writer::new();
    assert!(writer.spawn());
    writer.enqueue(&first, b"a".to_vec());
    writer.enqueue(&second, b"b".to_vec());
    assert!(writer.flush(Duration::from_secs(5)));
    assert_eq!(std::fs::read_to_string(&first).expect("read"), "a");
    assert_eq!(std::fs::read_to_string(&second).expect("read"), "b");
}
