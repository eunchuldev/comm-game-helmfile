import tokenizer

from parlai.scripts.train_model import TrainModel
TrainModel.main(
    task='fromfile:parlaiformat',
    #fromfile_datapath='dataset/dataset-normalized.txt',
    fromfile_datapath='t.txt',

    model_file='./model/test_train_90M',
    model='transformer/generator',
    multitask_weights=[1,3,3,3],

    dict_tokenizer='dc_jamo',

    embedding_size=512,
    n_layers=8,
    ffn_size=2048,
    dropout=0.1,
    n_heads=16,
    learn_positional_embeddings=True,
    n_positions=512,
    variant='xlm',
    activation='gelu',
    text_truncate=512,
    label_truncate=128,

    relu_dropout=0.0,
    save_after_valid=True,
    validation_metric_mode='min',
    validation_patience=15,
    validation_max_exs=20000,
    validation_metric='ppl',
    save_every_n_secs=60,


    learningrate=1e-06,
    optimizer='adam',
    lr_scheduler='reduceonplateau',
    gradient_clip=0.1,
    veps=0.25,
    betas=[0.9,0.999],
    update_freq=1,
    attention_dropout=0.0,
    skip_generation=True,
    #dynamic_batching='full',
    #fp16=True,
    #fp16_impl='mem_efficient',
    batchsize=8
)

#from parlai.scripts.display_data import DisplayData
#DisplayData.main(task='empathetic_dialogues', num_examples=5)
