mkdir -p dataset
gsutil rsync -r -d gs://comm-game/datalake/parlai/dcinside/document ./dataset
gzip -d -r -k -f dataset
cat dataset/*/*.txt > dataset-merged.txt
parlai display_data -t fromfile:parlaiformat --fromfile_datapath "dataset-merged.txt"
