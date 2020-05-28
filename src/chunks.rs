use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

fn split(chunk: u64, size: u64) -> Vec<u64> {
    let e = size / chunk + 1.min(size % chunk);
    (0..e).map(|i| i * chunk).collect()
}

#[derive(Debug)]
pub struct Chunk<C> {
    lines: std::io::Lines<C>,
    pos: u64,
    end: u64,
}

impl<C> Chunk<C> {
    pub fn new(
        mut c: C,
        chunk: usize,
        mut current: u64,
        start: u64,
        size: u64,
    ) -> anyhow::Result<(Self, u64)>
    where
        C: Seek + BufRead,
    {
        let skip = if current > start {
            true
        } else if start != current {
            c.seek(SeekFrom::Start(start - 1))?;
            let mut buf = [0 as u8; 1];
            if let Ok(1) = c.read(&mut buf) {
                buf[0] != b'\n'
            } else {
                false
            }
        } else {
            false
        };

        c.seek(SeekFrom::Start(start))?;
        current = if skip {
            let mut skip_leader = String::new();
            let _ = c.read_line(&mut skip_leader)?;
            start + skip_leader.len() as u64
        } else {
            start
        };
        let lines = c.lines();
        let c = Self {
            lines,
            pos: current,
            end: size.min(start + chunk as u64),
        };
        Ok((c, current))
    }
}

impl<T> Iterator for Chunk<T>
where
    T: Seek + BufRead,
{
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.pos >= self.end {
            return None;
        }
        match self.lines.next() {
            Some(Ok(l)) => {
                self.pos += l.len() as u64 + 1;
                Some(l)
            }
            _ => None,
        }
    }
}

pub struct Chunks<'a> {
    path: &'a Path,
    current: u64,
    starts: std::vec::IntoIter<u64>,
    chunk: usize,
    size: u64,
}

impl Chunks<'_> {
    pub fn new(path: &Path, chunk: usize, size: u64) -> Chunks {
        Chunks {
            path,
            current: 0,
            starts: split(chunk as u64, size).into_iter(),
            chunk,
            size,
        }
    }
}

impl Iterator for Chunks<'_> {
    type Item = Chunk<BufReader<File>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(start) = self.starts.next() {
            if let Ok(f) = File::open(&self.path) {
                let buf = BufReader::new(f);
                if let Ok((chunk, position)) =
                    Chunk::new(buf, self.chunk, self.current, start, self.size)
                {
                    self.current = position;
                    return Some(chunk);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use std::io::prelude::*;

    #[test]
    fn test_split() {
        assert_eq!(vec![0, 3, 6], split(3, 9));
        assert_eq!(vec![0, 3, 6, 9], split(3, 10));
    }

    #[test]
    fn test_chunks() {
        fn test_split_buf(i: Vec<String>, chunk_size: usize) -> TestResult {
            fn t(b: String, chunk_size: usize) -> anyhow::Result<()> {
                let dir = tempfile::tempdir()?;
                let path = dir.path().join("a");
                let mut file = File::create(&path)?;
                file.write_all(b.as_bytes())?;
                let chunks: Vec<_> = Chunks::new(&path, chunk_size, b.len() as u64)
                    .into_iter()
                    .map(|i| i.collect::<Vec<_>>().join("\n"))
                    .collect();
                let r = regex::Regex::new(r"\s+")?;
                let e = r.replace_all(&b, " ");
                let cs = chunks.join(" ");
                let a = r.replace_all(&cs, " ");
                if e.trim() == a.trim() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Expected >{}< Actual >{}<",
                        e.trim(),
                        a.trim()
                    ))
                }
            }

            let filter = regex::Regex::new(r"\W").unwrap();
            if chunk_size < 1 || i.iter().any(|s| filter.is_match(&s)) {
                return TestResult::discard();
            }
            if let Err(e) = t(i.join("\n"), chunk_size) {
                TestResult::error(format!("{:?}", e))
            } else {
                TestResult::from_bool(true)
            }
        }
        quickcheck::QuickCheck::new()
            .max_tests(100)
            .quickcheck(test_split_buf as fn(_, _) -> TestResult);
    }
}
