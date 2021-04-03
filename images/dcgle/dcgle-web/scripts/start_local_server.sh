#!/bin/sh

sh ./scripts/start_local_db.sh

docker build -t dcgle-web .

docker run -e DATABASE_URL="postgresql://dcgle:dcgle@localhost/dcgle" --net=host --rm -it dcgle-web
