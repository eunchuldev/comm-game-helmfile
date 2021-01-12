DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
docker build -q -t mysql-test $DIR
docker run -d --net=host --name mysql-test -p3306:3306 mysql-test
