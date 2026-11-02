//! Shared bounded, streaming line reader for IFC STEP and OBJ ingestion.
//! Keeps memory proportional to the configured line limit, not the input file size.
use std::io::BufRead;

/// Reads one complete line or returns a clear error before allocation exceeds maxBytes.
/// Caller can reuse destination across lines to avoid repeated heap allocations.
pub fn read_bounded_line<Reader: BufRead>(
  reader: &mut Reader,
  destination: &mut Vec<u8>,
  maxBytes: usize,
) -> Result<usize, String>
{
  destination.clear();

  loop
  {
    let available = reader
      .fill_buf()
      .map_err(|error| format!("Không đọc được dữ liệu: {error}"))?;

    if available.is_empty()
    {
      return Ok(destination.len());
    }

    let newlinePosition = available.iter().position(|byte| *byte == b'\n');
    let bytesToCopy = newlinePosition.map_or(available.len(), |index| index + 1);

    if destination.len().saturating_add(bytesToCopy) > maxBytes
    {
      return Err(format!("Dòng dữ liệu vượt giới hạn {maxBytes} bytes"));
    }

    destination.extend_from_slice(&available[..bytesToCopy]);
    reader.consume(bytesToCopy);

    if newlinePosition.is_some()
    {
      return Ok(destination.len());
    }
  }
}

#[cfg(test)]
mod tests
{
  use super::*;
  use std::io::{BufReader, Cursor};

  #[test]
  fn reads_lines_and_eof_with_reused_memory()
  {
    let mut reader = BufReader::with_capacity(2, Cursor::new(b"one\ntwo\n"));
    let mut buffer = Vec::new();
    assert_eq!(read_bounded_line(&mut reader, &mut buffer, 8).unwrap(), 4);
    assert_eq!(buffer, b"one\n");
    assert_eq!(read_bounded_line(&mut reader, &mut buffer, 8).unwrap(), 4);
    assert_eq!(buffer, b"two\n");
    assert_eq!(read_bounded_line(&mut reader, &mut buffer, 8).unwrap(), 0);
  }

  #[test]
  fn rejects_oversized_line_without_allocating_entire_input()
  {
    let input = vec![b'x'; 128 * 1024];
    let mut reader = BufReader::with_capacity(4096, Cursor::new(input));
    let mut buffer = Vec::new();
    let result = read_bounded_line(&mut reader, &mut buffer, 1024);
    assert!(result.is_err());
    assert!(buffer.len() <= 1024);
  }
}
