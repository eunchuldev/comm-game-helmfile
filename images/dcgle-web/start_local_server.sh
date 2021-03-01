#!/bin/sh
sh start_local_db.sh

cargo run -- --db-url=postgresql://dcgle:dcgle@localhost/dcgle
