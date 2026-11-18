#![cfg(feature = "std")]

use mayber::MaybeMut;
use std::io::{BufRead, Cursor, IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write};

#[test]
fn writes_and_seeks_reach_the_active_cursor() {
  let mut cursor = Cursor::new(Vec::new());
  for mut stream in [
    MaybeMut::Ref(&mut cursor),
    MaybeMut::Owned(Cursor::new(Vec::new())),
  ] {
    assert_eq!(stream.write(b"a").unwrap(), 1);
    assert_eq!(stream.write_vectored(&[IoSlice::new(b"b")]).unwrap(), 1);
    stream.write_all(b"c").unwrap();
    write!(stream, "{}", 4).unwrap();
    stream.flush().unwrap();
    assert_eq!(stream.stream_position().unwrap(), 4);
    stream.seek_relative(-2).unwrap();
    assert_eq!(stream.seek(SeekFrom::Current(1)).unwrap(), 3);
    stream.rewind().unwrap();
    let mut content = String::new();
    assert_eq!(stream.read_to_string(&mut content).unwrap(), 4);
    assert_eq!(content, "abc4");
  }
  assert_eq!(cursor.into_inner(), b"abc4");
}

#[test]
fn reads_and_buffered_operations_share_one_position() {
  let data = b"01234567,89;line\nrest";
  let mut cursor = Cursor::new(&data[..]);
  for mut stream in [
    MaybeMut::Ref(&mut cursor),
    MaybeMut::Owned(Cursor::new(&data[..])),
  ] {
    let mut buf = [0; 2];
    assert_eq!(stream.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"01");
    assert_eq!(
      stream
        .read_vectored(&mut [IoSliceMut::new(&mut buf)])
        .unwrap(),
      2
    );
    assert_eq!(&buf, b"23");
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"45");
    assert!(stream.fill_buf().unwrap().starts_with(b"67"));
    stream.consume(1);
    let mut output = Vec::new();
    assert_eq!(stream.read_until(b',', &mut output).unwrap(), 2);
    assert_eq!(output, b"7,");
    assert_eq!(stream.skip_until(b';').unwrap(), 3);
    let mut line = String::new();
    assert_eq!(stream.read_line(&mut line).unwrap(), 5);
    assert_eq!(line, "line\n");
    output.clear();
    assert_eq!(stream.read_to_end(&mut output).unwrap(), 4);
    assert_eq!(output, b"rest");
    assert_eq!(
      stream.read_exact(&mut buf).unwrap_err().kind(),
      std::io::ErrorKind::UnexpectedEof
    );
  }
}
