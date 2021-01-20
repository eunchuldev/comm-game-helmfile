#rm -rf model
mkdir -p model

parlai train \
  -t fromfile:parlaiformat --fromfile_datapath dataset-merged.txt \
  --model-file ./model/test_train_90M \
  -m transformer/generator \
  --multitask-weights 1,3,3,3 \
  --dict-tokenizer bpe \
  --dict-maxtokens 50000 \
  --embedding-size 512 --n-layers 8 --ffn-size 2048 --dropout 0.1 --n-heads 16 --learn-positional-embeddings True \
  --n-positions 512 --variant xlm --activation gelu --text-truncate 512 --label-truncate 128 -lr 1e-06 --optimizer adam \
  --lr-scheduler reduceonplateau --gradient-clip 0.1 -veps 0.25 --betas 0.9,0.999 --update-freq 1 --attention-dropout 0.0 \
  --relu-dropout 0.0 -vp 15 -stim 60 -vme 20000 -vmt ppl -vmm min --save-after-valid True \
  --dynamic-batching full \
  --skip-generation True --fp16 True --fp16-impl mem_efficient -bs 8

#python parlai/scripts/safe_interactive.py -t blended_skill_talk -mf zoo:blender/blender_90M/model

