docker run -d --rm --name dcgle-test-db --net=host -p 5432:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=postgres postgres:13 -c fsync=off

