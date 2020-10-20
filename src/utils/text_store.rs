use std::collections::HashSet;
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::str::FromStr;

pub struct TextStore<T>
where
    T: Eq + Hash + Display + FromStr,
{
    inner: HashSet<T>,
}

impl<T> TextStore<T>
where
    T: Eq + Hash + Display + FromStr,
{
    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn add(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    pub fn has(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.inner.remove(value)
    }

    pub fn synchronize(&self, path: &str) -> std::io::Result<()> {
        let chained_string = self.inner.iter().fold(String::from(""), |mut acc, value| {
            acc.push_str(&format!("{}\n", value));
            acc
        });

        fs::write(path, &chained_string)?;

        Ok(())
    }

    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let text = fs::read_to_string(path)?;

        if text.trim().is_empty() {
            return Ok(Self {
                inner: HashSet::new(),
            });
        }

        let inner = text
            .trim()
            .lines()
            .filter_map(|s| s.parse::<T>().ok())
            .collect::<HashSet<_>>();

        Ok(Self { inner })
    }
}
