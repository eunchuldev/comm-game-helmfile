#!/usr/bin/julia

using Test


const CHO  = ['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
const JUNG  = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
const JONG  = ['\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];

function _split_jamo(c::AbstractChar)
  if '가' <= c && c <= '힣' 
    i = Int(c)
    cho_index =  (i - 44032) ÷ 588
    jung_index =  (i - 44032 - cho_index * 588) ÷ 28
    jong_index =  i - 44032 - cho_index * 588 - jung_index * 28
    return [CHO[cho_index + 1], JUNG[jung_index + 1], JONG[jong_index + 1]]
  else 
    return [c, '\0', '\0']
  end
end

function split_jamo(text::String) 
  return collect(String, filter(x -> x != '\0', reduce(vcat, _split_jamo(char) for char in text)))
end

@test split_jamo("가나 a") == "가"
