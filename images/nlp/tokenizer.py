from parlai.core.dict import DictionaryAgent

from tokenizers import CharBPETokenizer

tokenizer = CharBPETokenizer(
    './dataset/vocab/vocab.json',
    './dataset/vocab/merges.txt')

def dc_tokenize(self, text):
    return tokenizer.encode(text).tokens

DictionaryAgent.dc_tokenize = dc_tokenize


CHO  = ['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
JUNG  = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
JONG  = ['\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
def split_jamo(c):
    if '가' <= c <= '힣':
        i = ord(c)
        cho_index =  (i - 44032) // 588
        jung_index =  (i - 44032 - cho_index * 588) // 28
        jong_index =  i - 44032 - cho_index * 588 - jung_index * 28
        yield CHO[cho_index]
        yield JUNG[jung_index]
        if jong_index: yield JONG[jong_index]
    else:
        yield c

jamo_tokenizer = CharBPETokenizer(
    './dataset/vocab-jamo/vocab.json',
    './dataset/vocab-jamo/merges.txt')

def dc_jamo_tokenize(self, text):
    return jamo_tokenizer.encode(''.join(t for c in text for t in split_jamo(c))).tokens

DictionaryAgent.dc_jamo_tokenize = dc_jamo_tokenize

if __name__== '__main__':
    import sys
    print(dc_jamo_tokenize(None, sys.argv[1]))
