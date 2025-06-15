#!/bin/sh

DEV_IP=167.99.247.110
WORKDIR=$(mktemp -d)
cd $WORKDIR

git clone http://heracles:3000/chance/game_test.git

tar -czf game.tar.gz game_test
scp game.tar.gz root@$DEV_IP:~/
ssh root@$DEV_IP "rm -rf ~/game_test"
ssh root@$DEV_IP "tar -xf ./game.tar.gz"
ssh root@$DEV_IP "cd ~/game_test && docker build . -t game_test:latest"
