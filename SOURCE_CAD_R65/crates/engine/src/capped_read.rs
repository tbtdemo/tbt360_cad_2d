//! Bounded reads of local files.
//!
//! Reading a path with [`std::fs::read`] buffers until EOF, so a path naming an endless readable
//! device (for example `/dev/urandom` or a FIFO) exhausts process memory before any size check
//! runs (CWE-400). [`read_capped`] opens the path once, refuses anything that is not a regular
//! file, and reads through a reader limited to `max + 1` bytes so oversized input is rejected
//! before it is fully buffered.

#![cfg(not(target_arch = "wasm32"))]

use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;

/// Read at most `max` bytes from the regular file at `path`.
///
/// Returns an error if the path is not a regular file, cannot be opened, or holds more than `max`
/// bytes. At most `max + 1` bytes are ever buffered, so an endless device cannot exhaust memory.
pub fn read_capped(path: &Path, max: usize) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a regular file"));
    }
    // Read one byte past the limit so we can tell "exactly max" from "too big" without trusting
    // the metadata length (which a device may misreport).
    let mut buf = Vec::new();
    let cap = (max as u64).saturating_add(1);
    let read = file.take(cap).read_to_end(&mut buf)?;
    if read as u64 > max as u64 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("file is larger than the {max}-byte limit")));
    }
    Ok(buf)
}


/// Retry transient interrupted system reads rather than turning an OS signal
/// into a failed CAD open. This mirrors Read::read_to_end semantics without
/// allocating a second large buffer.
fn read_retry_interrupted(reader: &mut impl Read, buf: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buf) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

/// Progressive read for large regular drawing files. Unlike read_capped, this API does not
/// impose a fixed on-disk file-size ceiling. Each allocation is fallible and chunked.
/// A file may still fail to decode if actual memory is insufficient; callers must report that.
pub fn read_progressive(path: &Path, mut report: impl FnMut(u64, u64)) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let mut file = options.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() { return Err(io::Error::new(io::ErrorKind::InvalidInput, "not a regular file")); }
    let total = meta.len();
    let mut output = Vec::new();
    // Reserve only a small bounded hint, not the whole drawing length. This avoids
    // repeated tiny reallocations for ordinary drawings without committing hundreds
    // of MiB before the file is actually read.
    let reserve_hint = total.min(8 * 1024 * 1024) as usize;
    output.try_reserve(reserve_hint).map_err(|e| io::Error::other(format!("not enough memory to initialize drawing read: {e}")))?;
    // Read directly into the destination Vec rather than allocating a separate
    // 1 MiB scratch buffer and copying every byte a second time. Keep growth
    // fallible and bounded per iteration; there is still no fixed file-size cap.
    const READ_BLOCK: usize = 1024 * 1024;
    let mut read_total = 0u64;
    report(0, total);
    loop {
        let old_len = output.len();
        // Most regular CAD files have reliable metadata. Avoid zero-initializing
        // a full MiB for a tiny trailing chunk; still permit growth beyond the
        // observed metadata length by probing another block at the boundary.
        let remaining = total.saturating_sub(read_total);
        let read_size = if remaining == 0 { READ_BLOCK } else { remaining.min(READ_BLOCK as u64) as usize };
        output.try_reserve(read_size).map_err(|e| io::Error::other(format!("not enough memory to read drawing: {e}")))?;
        output.resize(old_len + read_size, 0);
        let n = match read_retry_interrupted(&mut file, &mut output[old_len..]) {
            Ok(n) => n,
            Err(e) => {
                output.truncate(old_len);
                return Err(e);
            }
        };
        output.truncate(old_len + n);
        if n == 0 { break; }
        read_total = read_total.saturating_add(n as u64);
        report(read_total, total);
    }
    report(read_total, read_total);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_interrupted_read_retries_without_losing_geometry_bytes() {
        struct OneInterruptedRead { data: Vec<u8>, interrupted: bool, cursor: usize }
        impl Read for OneInterruptedRead {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "transient signal"));
                }
                let remaining = &self.data[self.cursor..];
                let count = remaining.len().min(out.len());
                out[..count].copy_from_slice(&remaining[..count]);
                self.cursor += count;
                Ok(count)
            }
        }
        let mut reader = OneInterruptedRead { data: vec![1, 2, 3, 4], interrupted: false, cursor: 0 };
        let mut bytes = [0; 4];
        assert_eq!(read_retry_interrupted(&mut reader, &mut bytes).unwrap(), 4);
        assert_eq!(bytes, [1, 2, 3, 4]);
    }

    /// Regular-file reads accept the byte boundary and reject any larger payload.
    #[test]
    fn reads_within_limit_and_rejects_oversize() {
        let dir = std::env::temp_dir().join(format!("cadcraft-cr-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("f.bin");
        std::fs::write(&p, vec![7u8; 100]).unwrap();
        assert_eq!(read_capped(&p, 100).unwrap().len(), 100);
        assert_eq!(read_capped(&p, 200).unwrap().len(), 100);
        assert!(read_capped(&p, 99).is_err());
        assert_eq!(read_capped(&p, usize::MAX).unwrap().len(), 100);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An endless device is rejected before its contents can be buffered.
    #[cfg(unix)]
    #[test]
    fn rejects_non_regular_file() {
        // /dev/zero is an endless readable device; it must be rejected, not buffered.
        let p = Path::new("/dev/zero");
        if p.exists() {
            assert!(read_capped(p, 8 << 20).is_err());
        }
    }

    #[test]
    fn progressive_read_reports_real_byte_progress_and_accepts_regular_file() {
        let dir = std::env::temp_dir().join(format!("cadcraft-progress-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("drawing.dxf");
        let data = vec![42u8; 3 * 1024 * 1024 + 3];
        std::fs::write(&path, &data).unwrap();
        let mut updates = Vec::new();
        let result = read_progressive(&path, |done, total| updates.push((done, total))).unwrap();
        assert_eq!(result, data);
        assert!(updates.len() >= 4);
        assert_eq!(updates.first().copied(), Some((0, data.len() as u64)));
        assert_eq!(updates.last().copied(), Some((data.len() as u64, data.len() as u64)));
        assert!(updates.windows(2).all(|pair| pair[0].0 <= pair[1].0));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn progressive_read_preserves_partial_tail_without_zero_padding() {
        let dir = std::env::temp_dir().join(format!("cadcraft-progress-tail-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tail.dxf");
        let mut original = vec![0xA5u8; 2 * 1024 * 1024 + 17];
        original[0] = 0x01;
        *original.last_mut().unwrap() = 0xFE;
        std::fs::write(&path, &original).unwrap();
        let mut last = (0, 0);
        let read = read_progressive(&path, |done, total| last = (done, total)).unwrap();
        assert_eq!(read, original);
        assert_eq!(last, (original.len() as u64, original.len() as u64));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn progressive_empty_file_reports_zero_and_returns_empty_data() {
        let dir = std::env::temp_dir().join(format!("cadcraft-progress-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.dxf");
        std::fs::write(&path, []).unwrap();
        let mut updates = Vec::new();
        let content = read_progressive(&path, |done, total| updates.push((done, total))).unwrap();
        assert!(content.is_empty());
        assert!(updates.iter().all(|pair| *pair == (0, 0)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Opening a FIFO without a writer returns an error without waiting for a peer.
    #[cfg(unix)]
    #[test]
    fn rejects_fifo_without_a_writer() {
        let dir = std::env::temp_dir().join(format!("cadcraft-cr-fifo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("pipe");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        let started = std::time::Instant::now();
        assert_eq!(read_capped(&fifo, 1024).unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
