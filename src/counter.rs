use ahash::AHashMap;

#[derive(Default)]
pub struct Counter {
    counts: AHashMap<String, u64>,
    top: AHashMap<String, u64>,
    threshold: u64,
    num: usize,
}

impl Counter {
    pub fn new(num: usize) -> Self {
        Self {
            num,
            ..Default::default()
        }
    }

    pub fn counts(self) -> HashMap<String, u64> {
        self.counts
    }

    pub fn add(&mut self, key: &str, added: u64) {
        let count = match self.counts.get_mut(&*key) {
            Some(count) => {
                *count += added;
                *count
            }
            None => {
                self.counts.insert((&*key).to_owned(), added);
                added
            }
        };

        if count < self.threshold || self.num == 0 {
            return;
        }
        self.top.insert(key.into(), count);

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
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyCount {
    pub count: u64,
    pub key: String,
}
