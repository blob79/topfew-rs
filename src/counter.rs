use ahash::AHashMap;

#[derive(Default)]
pub struct Counter {
    counts: AHashMap<String, u64>,
    top: AHashMap<String, u64>,
    threshold: u64,
    num: usize,
}

impl Counter {
    pub fn new(num: Option<usize>) -> Self {
        Self {
            num: num.unwrap_or(0),
            threshold: num.map(|_| 0).unwrap_or(u64::MAX),
            ..Default::default()
        }
    }

    pub fn add(&mut self, key: &str, added: u64) {
        let count = match self.counts.get_mut(key) {
            Some(count) => {
                *count += added;
                *count
            }
            None => {
                self.counts.insert(key.to_owned(), added);
                added
            }
        };

        if count < self.threshold {
            return;
        }

        if let Some(t) = self.top.get_mut(key) {
            *t = count;
            return;
        }

        self.top.insert(key.to_owned(), count);
        if self.top.len() < self.num * 2 {
            return;
        }

        let mut top_values = self.top.values().collect::<Vec<_>>();
        top_values.sort_unstable();
        let threshold = *top_values[self.num as usize - 1];
        self.threshold = threshold;
        self.top.retain(|_, v| *v > threshold);
    }

    pub fn top(&self) -> Vec<KeyCount> {
        let mut top = Vec::with_capacity(self.num);
        for (key, &count) in &self.top {
            top.push(KeyCount {
                count,
                key: key.into(),
            });
        }

        top.sort_unstable();
        top.reverse();
        top
    }

    pub fn merge(&mut self, r: Counter) {
        for (key, count) in r.counts.iter() {
            self.add(key, *count);
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyCount {
    pub count: u64,
    pub key: String,
}
