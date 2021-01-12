if [[ -n "$(docker ps | grep mysql-test)" ]]; then
  docker stop mysql-test
  docker rm mysql-test
fi
if [[ -n "$(docker ps -al | grep mysql-test)" ]]; then
  docker rm mysql-test
fi
