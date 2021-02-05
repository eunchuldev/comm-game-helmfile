#rm -rf model
mkdir -p model

parlai interactive -m transformer/generator -mf "./model/test_train_90M"
