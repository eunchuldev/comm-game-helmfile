mkdir -p dataset
gsutil rsync -r -d gs://datalake-cg/datalake/parlai/dcinside/document ./dataset
gzip -d -r -k -f dataset
cat dataset/*/*.txt > dataset-merged.txt
parlai display_data -t fromfile:parlaiformat --fromfile_datapath "dataset-merged.txt"
