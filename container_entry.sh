#!/bin/bash

set -e

service nginx start

cd /front-server && FRONT_PORT=8999 ./server &

cd /server && ./server --addr=127.0.0.1:8998 &

echo 'starting: press enter to kill' && read -p ''
