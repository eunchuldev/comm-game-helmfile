mkdir -p dataset/shards
gsutil rsync -r -d gs://comm-game/datalake/parlai-skip-non-text/dcinside/document ./dataset/shards
gzip -d -r -k -f dataset/shards
cat dataset/shards/*/*.txt > dataset/dataset-merged.txt
echo normalize..
./util/norm/target/release/norm -w -c _ -r 5 -i dataset/dataset-merged.txt -o dataset/dataset-normalized.txt
echo normalize..(jamo split)
./util/norm/target/release/norm -wh -c _ -r 5 -i dataset/dataset-merged.txt -o dataset/dataset-normalized-jamo.txt
echo gen tokenize dataset..
./util/parlai-dataset-to-tokenizer-dataset/main.jl dataset/dataset-normalized.txt dataset/tokenizer-dataset.txt
echo gen tokenize dataset..(jamo split)
./util/parlai-dataset-to-tokenizer-dataset/main.jl dataset/dataset-normalized-jamo.txt dataset/tokenizer-dataset-jamo.txt
echo train tokenizer..
./util/tokenize/train.py dataset/tokenizer-dataset.txt --vocab_size=30000 --vocab_path="dataset/vocab"
./util/tokenize/train.py dataset/tokenizer-dataset-jamo.txt --vocab_size=30000 --vocab_path="dataset/vocab-jamo"
#parlai display_data -t fromfile:parlaiformat --fromfile_datapath "dataset/dataset-merged-normalized.txt"
