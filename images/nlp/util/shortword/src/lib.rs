use lazy_static::lazy_static;
use clap::Clap;
use regex::Regex;
use regex::Replacer;


use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use std::borrow::Cow;

use serde_json;

#[derive(Clap)]
#[clap(version = "1.0", author = "Eunchul <eunchul.dev@gmail.com>")]
pub struct Opts {
    #[clap(short, long)]
    pub input_file_path: Option<String>,
    #[clap(short, long)]
    pub output_file_path: Option<String>,
    #[clap(short, long, min_values=1)]
    pub reg: Vec<String>,
    #[clap(short, long, min_values=1)]
    pub sub: Vec<String>,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
}

fn hash<T: Hash>(t: &T, cardinality: u64) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish() % cardinality
}

pub struct WordShorter {
    pub replace_rules: Vec<(Regex, String)>,
    pub dict: HashMap<String, String>,
}

impl WordShorter {
    fn new() -> Self {
        WordShorter{
            replace_rules: Vec::new(),
            dict: HashMap::new(),
        }
    }
    fn with_rules<'a>(mut self, rules: impl std::iter::IntoIterator<Item=(&'a str, &'a str)>) -> Self {
        self.replace_rules = rules.into_iter().map(|(r, s)| (Regex::new(r).unwrap(), s.to_string())).collect();
        self
    }
    fn load(mut self, path: &str) -> anyhow::Result<Self> {
        self.dict = serde_json::from_slice(&std::fs::read(path)?)?;
        Ok(self)
    }
    fn save(mut self, path: &str) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer(file, &self.dict)?;
        Ok(())
    }
    pub fn short<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        self.replace_rules.iter().fold::<Cow<str>, _>(
            text,
            |text: Cow<str>, (rule, replacer)|  match text {
                Cow::Borrowed(s) => rule.replace_all(s, replacer.as_str().by_ref()),
                Cow::Owned(s) => Cow::Owned(rule.replace_all(&s, replacer.as_str().by_ref()).into_owned()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_shorts() {
        let ws = WordShorter::new().with_rules(vec![("a", "b")]);
        let res = ws.short("bbbabbbb".into());
        let expected = "bbbbbbbb";
        assert_eq!(&res, expected);
        assert_eq!(ws.dict, HashMap::from_iter(vec![()]));
    }
}
