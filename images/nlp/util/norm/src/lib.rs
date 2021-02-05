use lazy_static::lazy_static;
use clap::Clap;
use regex::Regex;


#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
pub struct Opts {
    #[clap(short, long)]
    pub input_file_path: Option<String>,
    #[clap(short, long)]
    pub output_file_path: Option<String>,
    #[clap(short, long)]
    pub hangul_to_jamo: bool,
    #[clap(short, long)]
    pub control_chars: Option<String>,
    #[clap(short, long)]
    pub repeat: Option<usize>,
    #[clap(short, long)]
    pub whitespace_less: bool,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
}

pub fn control_chars(text: String, replacer: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^A-Za-z0-9ㄱ-ㅎㅏ-ㅣ가-힣~!?.,();*/=+\-\[\]\s\n]").unwrap();
    }
    RE.replace_all(&text, replacer).into_owned()
}

pub fn derepeat(text: String, n: usize) -> String{
    let mut last_char: char = '𝕊';
    let mut repeat: usize = 0;
    text.chars().filter_map(|c| {
        if last_char == c {
            repeat += 1;
        } else {
            repeat = 0;
            last_char = c;
        }
        if repeat >= n {
            None
        } else {
            Some(c)
        }
    }).collect()
}

pub fn whitespace_less(text: String) -> String {
    let mut last_char: char = '𝕊';
    text.trim().chars().filter_map(|c| {
        if char::is_whitespace(last_char) && char::is_whitespace(c) {
            None
        } else {
            last_char = c;
            Some(c)
        }
    }).collect()
}

pub fn normalize<'a>(text: String, opts: &'a Opts) -> String {
    let text = match &opts.control_chars {
        Some(c) => control_chars(text, &c),
        None => text,
    };
    let text = match opts.repeat {
        Some(n) => derepeat(text, n),
        None => text,
    };
    let text = match &opts.whitespace_less {
        true => whitespace_less(text),
        false => text,
    };
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_whitespace_less() {
        assert_eq!(whitespace_less("   가     나  다 라    ".to_string()), "가 나 다 라".to_string());
    }
    #[test]
    fn it_control_chars() {
        assert_eq!(control_chars("가힣#ㄱㅏz1()!?[]/ &".to_string(), "흠"), "가힣흠ㄱㅏz1()!?[]/ 흠".to_string());
    }
    #[test]
    fn it_derepeat() {
        assert_eq!(derepeat("아아아아아 음음음 호호호호 홀홀 ".to_string(), 3), "아아아 음음음 호호호 홀홀 ".to_string());
    }

}
