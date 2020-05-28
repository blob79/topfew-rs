use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;

mod chunks;
mod counter;
mod key_finder;

use chunks::Chunks;
pub use counter::{Counter, KeyCount};
pub use key_finder::KeyFinder;

fn sum(mut l: Counter, r: Counter) -> Counter {
    for (key, count) in r.counts().into_iter() {
        l.add(&key, count);
    }
    l
}

pub fn top_few_from_stream(
    path: &Path,
    kf: &KeyFinder,
    num: usize,
) -> anyhow::Result<Vec<KeyCount>> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();

    let total = Chunks::new(path, 1 << 26, size)
        .par_bridge()
        .map(|reader| {
            let mut counter = Counter::new(0);
            let mut s = String::new();
            for ln in reader {
                s.clear();
                if let Ok(key) = kf.key(&ln, &mut s) {
                    counter.add(key, 1)
                }
            }
            counter
        })
        .fold(|| Counter::new(0), sum)
        .reduce(|| Counter::new(num), sum);

    Ok(total.top())
}
