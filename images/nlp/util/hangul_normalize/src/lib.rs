use lazy_static::lazy_static;
use clap::Clap;
use regex::Regex;
use std::iter;



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
        static ref RE: Regex = Regex::new(r"[^A-Za-z0-9ㄱ-ㅎㅏ-ㅣ가-힣~!?.,():;*/=+\-\[\]\s\n<>]").unwrap();
    }
    RE.replace_all(&text, replacer).into_owned()
}


pub fn hangul_to_jamo(text: String) -> String {
    const CHO: [char; 19]  = ['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
    const JUNG: [char; 21]  = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
    const JONG: [char; 28]  = ['\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
    text.chars().flat_map(|c| {
        if '가' <= c && c <= '힣' {
            let c = c as usize;
            let cho_index =  (c - 44032) / 588;
            let jung_index =  (c - 44032 - cho_index * 588) / 28;
            let jong_index =  c - 44032 - cho_index * 588 - jung_index * 28;
            iter::once(CHO[cho_index]).chain(iter::once(JUNG[jung_index])).chain(iter::once(JONG[jong_index]))
        } else {
            iter::once(c).chain(iter::once('\0')).chain(iter::once('\0'))
        }
    }).filter(|c| c != &'\0').collect()
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
            if c == '\t' {
                Some(c)
            } else {
                None
            }
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
    let text = match &opts.hangul_to_jamo {
        true => hangul_to_jamo(text),
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
    fn it_hangul_to_jamo() {
        assert_eq!(hangul_to_jamo("가힣 뷁 ab123킼ㄱㄴㄷ".to_string()), "ㄱㅏㅎㅣㅎ ㅂㅞㄺ ab123ㅋㅣㅋㄱㄴㄷ".to_string());
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
