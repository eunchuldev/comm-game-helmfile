DATABASE_URL="postgresql://dcgle:dcgle@localhost/dcgle" sqlx database create
DATABASE_URL="postgresql://postgres:postgres@localhost/postgres" sqlx database create
cd model && DATABASE_URL="postgres://postgres:postgres@localhost/postgres" sqlx migrate run
DATABASE_URL="postgresql://postgres:postgres@localhost/postgres" cargo test --all 
#docker build -t dcgle-test . -f Dockerfile.test
