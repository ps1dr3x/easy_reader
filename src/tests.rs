use super::*;
use std::fs::File;

#[test]
fn test_empty_file() {
    let file = File::open("resources/empty-file").unwrap();
    let reader = EasyReader::new(file);

    assert!(
        reader.is_err(),
        "Empty file, but the constructor hasn't returned an Error"
    );
}

#[test]
fn test_one_line_file() {
    let file = File::open("resources/one-line-file").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    assert!(
        reader.next_line().unwrap().unwrap().eq("A"),
        "The single line of one-line-file should be: A"
    );
    assert!(
        reader.next_line().unwrap().is_none(),
        "There is no other lines in one-line-file, this should be None"
    );
    assert!(
        reader.prev_line().unwrap().is_none(),
        "There is no other lines in one-line-file, this should be None"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("A"),
        "The single line of one-line-file should be: A"
    );

    reader.bof();
    assert!(
        reader.next_line().unwrap().unwrap().eq("A"),
        "The single line of one-line-file from the bof should be: A"
    );

    reader.eof();
    assert!(
        reader.prev_line().unwrap().unwrap().eq("A"),
        "The single line of one-line-file from the eof should be: A"
    );

    #[cfg(feature = "rand")]
    for _i in 1..10 {
        assert!(
            reader.random_line().unwrap().unwrap().eq("A"),
            "The single line of one-line-file should be: A (test: 10 random lines)"
        );
    }
}

#[test]
fn test_move_through_lines() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    reader.eof();
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "[test-file-lf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader
            .next_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.bof();
    assert!(
        reader.next_line().unwrap().unwrap().eq("AAAA AAAA"),
        "[test-file-lf] The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-lf] The second line from the BOF should be: B B BB BBB"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the BOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-lf] The second line from the BOF should be: B B BB BBB"
    );

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    reader.eof();
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "[test-file-crlf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader
            .next_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.bof();
    assert!(
        reader.next_line().unwrap().unwrap().eq("AAAA AAAA"),
        "[test-file-crlf] The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-crlf] The second line from the BOF should be: B B BB BBB"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the BOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-crlf] The second line from the BOF should be: B B BB BBB"
    );
}

#[cfg(feature = "rand")]
#[test]
fn test_random_line() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    for _i in 0..100 {
        let random_line = reader.random_line().unwrap().unwrap();
        assert!(
            !random_line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    for _i in 0..100 {
        let random_line = reader.random_line().unwrap().unwrap();
        assert!(
            !random_line.is_empty(),
            "Empty line, but test-file-crlf does not contain empty lines"
        );
    }
}

#[test]
fn test_iterations() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();

    while let Ok(Some(line)) = reader.next_line() {
        assert!(
            !line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }
    assert!(
        reader.current_end_line_offset == reader.file_size,
        "After the \"while next-line\" iteration the offset should be at the EOF"
    );
    assert!(
        reader
            .current_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.eof();
    while let Ok(Some(line)) = reader.prev_line() {
        assert!(
            !line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }
    assert!(
        reader.current_start_line_offset == 0,
        "After the \"while prev-line\" iteration the offset should be at the BOF"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("AAAA AAAA"),
        "The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "The second line from the BOF should be: B B BB BBB"
    );
}

#[test]
fn test_indexed() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.build_index().unwrap();

    reader.eof();
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "[test-file-lf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader
            .next_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.bof();
    assert!(
        reader.next_line().unwrap().unwrap().eq("AAAA AAAA"),
        "[test-file-lf] The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-lf] The second line from the BOF should be: B B BB BBB"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the BOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-lf] The second line from the BOF should be: B B BB BBB"
    );

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.build_index().unwrap();

    reader.eof();
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "[test-file-crlf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader
            .next_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.bof();
    assert!(
        reader.next_line().unwrap().unwrap().eq("AAAA AAAA"),
        "[test-file-crlf] The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-crlf] The second line from the BOF should be: B B BB BBB"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the BOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"),
        "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC"
    );
    assert!(
        reader.prev_line().unwrap().unwrap().eq("B B BB BBB"),
        "[test-file-crlf] The second line from the BOF should be: B B BB BBB"
    );

    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.build_index().unwrap();

    while let Ok(Some(line)) = reader.next_line() {
        assert!(
            !line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }
    assert!(
        reader.current_end_line_offset == reader.file_size,
        "After the \"while next-line\" iteration the offset should be at the EOF"
    );
    assert!(
        reader
            .current_line()
            .unwrap()
            .unwrap()
            .eq("EEEE  EEEEE  EEEE  EEEEE"),
        "The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE"
    );
    assert!(
        reader
            .prev_line()
            .unwrap()
            .unwrap()
            .eq("DDDD  DDDDD DD DDD DDD DD"),
        "The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD"
    );

    reader.eof();
    while let Ok(Some(line)) = reader.prev_line() {
        assert!(
            !line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }
    assert!(
        reader.current_start_line_offset == 0,
        "After the \"while prev-line\" iteration the offset should be at the BOF"
    );
    assert!(
        reader.current_line().unwrap().unwrap().eq("AAAA AAAA"),
        "The first line from the BOF should be: AAAA AAAA"
    );
    assert!(
        reader.next_line().unwrap().unwrap().eq("B B BB BBB"),
        "The second line from the BOF should be: B B BB BBB"
    );

    let file = File::open("resources/test-file-lf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.build_index().unwrap();

    #[cfg(feature = "rand")]
    for _i in 0..100 {
        let random_line = reader.random_line().unwrap().unwrap();
        assert!(
            !random_line.is_empty(),
            "Empty line, but test-file-lf does not contain empty lines"
        );
    }

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.build_index().unwrap();

    #[cfg(feature = "rand")]
    for _i in 0..100 {
        let random_line = reader.random_line().unwrap().unwrap();
        assert!(
            !random_line.is_empty(),
            "Empty line, but test-file-crlf does not contain empty lines"
        );
    }
}

#[test]
fn test_file_with_blank_line_at_the_beginning() {
    let file = File::open("resources/file-with-blank-line-at-the-beginning").unwrap();
    let mut reader = EasyReader::new(file).unwrap();
    reader.eof();

    assert_eq!(
        reader.prev_line().unwrap().unwrap(),
        "Blank line above!".to_string(),
        "The last line should be: Blank line above!",
    );
    assert!(
        reader.prev_line().unwrap().unwrap().is_empty(),
        "The blank line at the BOF should be empty"
    );
    assert!(
        reader.prev_line().unwrap().is_none(),
        "The file should only have two lines"
    );
}
