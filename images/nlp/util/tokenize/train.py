#!/usr/bin/python

import fire
from tokenizers import CharBPETokenizer

def train(
    path, 
    vocab_size=30000, 
    min_frequency=5,
    vocab_path='.'):
    tokenizer = CharBPETokenizer()
    tokenizer.train([path], vocab_size=vocab_size, min_frequency=min_frequency)
    tokenizer.save_model(vocab_path)

if __name__ == '__main__':
    fire.Fire(train)
