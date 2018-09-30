use super::*;

#[test]
fn test_empty_file() {
    let file = File::open("resources/empty-file").unwrap();
    let easy_reader: Result<EasyReader, Error> = EasyReader::new(file);

    assert!(easy_reader.is_err(), "Empty file, but the constructor hasn't returned an Error");
}

#[test]
fn test_one_line_file() {
    let file = File::open("resources/one-line-file").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    assert!(easy_reader.next_line().unwrap().unwrap().eq("A"), "The single line of one-line-file should be: A");
    assert!(easy_reader.next_line().unwrap().is_none(), "There is no other lines in one-line-file, this should be None");
    assert!(easy_reader.prev_line().unwrap().is_none(), "There is no other lines in one-line-file, this should be None");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("A"), "The single line of one-line-file should be: A");
    
    easy_reader.from_bof();
    assert!(easy_reader.next_line().unwrap().unwrap().eq("A"), "The single line of one-line-file from the bof should be: A");

    easy_reader.from_eof();
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("A"), "The single line of one-line-file from the eof should be: A");

    for _i in 1..10 {
        assert!(easy_reader.random_line().unwrap().unwrap().eq("A"), "The single line of one-line-file should be: A (test: 10 random lines)");
    }
}

#[test]
fn test_move_through_lines() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    easy_reader.from_eof();
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "[test-file-lf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-lf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

    easy_reader.from_bof();
    assert!(easy_reader.next_line().unwrap().unwrap().eq("AAAA AAAA"), "[test-file-lf] The first line from the BOF should be: AAAA AAAA");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("B B BB BBB"), "[test-file-lf] The second line from the BOF should be: B B BB BBB");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the BOF should be: CCCC  CCCCC");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-lf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("B B BB BBB"), "[test-file-lf] The second line from the BOF should be: B B BB BBB");

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    easy_reader.from_eof();
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "[test-file-crlf] The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "[test-file-crlf] The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

    easy_reader.from_bof();
    assert!(easy_reader.next_line().unwrap().unwrap().eq("AAAA AAAA"), "[test-file-crlf] The first line from the BOF should be: AAAA AAAA");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("B B BB BBB"), "[test-file-crlf] The second line from the BOF should be: B B BB BBB");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the BOF should be: CCCC  CCCCC");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("CCCC  CCCCC"), "[test-file-crlf] The third line from the EOF should be: CCCC  CCCCC");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("B B BB BBB"), "[test-file-crlf] The second line from the BOF should be: B B BB BBB");
}

#[test]
fn test_random_line() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    for _i in 0..100 {
        let random_line = easy_reader.random_line().unwrap().unwrap();
        assert!(!random_line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
    }

    let file = File::open("resources/test-file-crlf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    for _i in 0..100 {
        let random_line = easy_reader.random_line().unwrap().unwrap();
        assert!(!random_line.is_empty(), "Empty line, but test-file-crlf does not contain empty lines");
    }
}

#[test]
fn test_iterations() {
    let file = File::open("resources/test-file-lf").unwrap();
    let mut easy_reader = EasyReader::new(file).unwrap();

    while let Ok(Some(line)) = easy_reader.next_line() {
        assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
    }
    assert!(easy_reader.current_end_line_offset == easy_reader.file_size, "After the \"while next-line\" iteration the offset should be at the EOF");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("EEEE  EEEEE  EEEE  EEEEE"), "The first line from the EOF should be: EEEE  EEEEE  EEEE  EEEEE");
    assert!(easy_reader.prev_line().unwrap().unwrap().eq("DDDD  DDDDD DD DDD DDD DD"), "The second line from the EOF should be: DDDD  DDDDD DD DDD DDD DD");

    easy_reader.from_eof();
    while let Ok(Some(line)) = easy_reader.prev_line() {
        assert!(!line.is_empty(), "Empty line, but test-file-lf does not contain empty lines");
    }
    assert!(easy_reader.current_start_line_offset == 0, "After the \"while prev-line\" iteration the offset should be at the BOF");
    assert!(easy_reader.current_line().unwrap().unwrap().eq("AAAA AAAA"), "The first line from the BOF should be: AAAA AAAA");
    assert!(easy_reader.next_line().unwrap().unwrap().eq("B B BB BBB"), "The second line from the BOF should be: B B BB BBB");
}
