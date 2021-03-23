#!/bin/sh
NAME=test-pg
docker stop $NAME
sleep 1
docker run -e POSTGRES_USER=dcgle -e POSTGRESQL_DATABASE=dcgle -e POSTGRESQL_POSTGRES_PASSWORD=pg -e POSTGRES_PASSWORD=dcgle -e POSTGRESQL_PASSWORD=dcgle --net=host --name $NAME -d --rm -it postgres:13
sleep 3
#docker exec -e PGPASSWORD="pg" -it $NAME psql -U postgres dcgle -c "CREATE EXTENSION rum"
