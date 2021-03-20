#!/bin/sh
NAME=test-pg
docker stop $NAME
sleep 1
docker run -e POSTGRESQL_PASSWORD=dcgle -e POSTGRESQL_USERNAME=dcgle -e POSTGRESQL_DATABASE=dcgle -e POSTGRESQL_POSTGRES_PASSWORD=pg -p 5432:5432 --name $NAME -d --rm -it postgresql
sleep 3
docker exec -e PGPASSWORD="pg" -it $NAME psql -U postgres dcgle -c "CREATE EXTENSION rum"
