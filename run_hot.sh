#!/bin/bash

function prefix() {
    awk '{print "'$(tput setab $1)"[$2]"$(tput sgr0)' " $0}'
}
function prefix_deps() {
    prefix 2 "deps"
}
function prefix_fwd() {
    prefix 3 "fwd"
}
function prefix_front() {
    prefix 4 "front"
}
function prefix_client() {
    prefix 5 "client"
}
function prefix_server() {
    prefix 6 "server"
}
function prefix_renderer() {
    prefix 7 "renderer"
}

deps_fail=false

which tmux &> /dev/null
if [[ $? != 0 ]]; then
    echo "requires tmux" | prefix_deps
    deps_fail=true
fi

which deno &> /dev/null
if [[ $? != 0 ]]; then
    echo "requires deno" | prefix_deps
    deps_fail=true
fi

which npm &> /dev/null
if [[ $? != 0 ]]; then
    echo "requires npm" | prefix_deps
    deps_fail=true
fi

which inotifywait &> /dev/null
if [[ $? != 0 ]]; then
    echo "requires inotify-tools" | prefix_deps
    deps_fail=true
fi

which wasm-pack &> /dev/null
if [[ $? != 0 ]]; then
    echo "requires wasm-pack" | prefix_deps
    deps_fail=true
fi

if [ $deps_fail = true ]; then
    echo "deps check failed" | prefix_deps
    exit 1
fi

echo "deps ok" | prefix_deps

mkdir tmp &> /dev/null
mkdir front/built &> /dev/null

if [[ $1 == "" ]]; then
    tmux \
        new-session  './run_hot.sh server' \; \
        split-window -h './run_hot.sh client' \; \
        select-pane -L \; \
        split-window './run_hot.sh renderer' \; \
        select-pane -R \; \
        split-window './run_hot.sh front' \; \
        split-window -h './run_hot.sh forward'
elif [[ $1 == "forward" ]]; then
    pushd ./forward &> /dev/null

    docker build . \
        -f Dockerfile.hot \
        -t woods-hot-forward:latest | prefix_fwd

    echo "booting forward proxy" | prefix_fwd

    docker run \
        -p 80:80 \
        --add-host host.docker.internal:host-gateway \
        woods-hot-forward:latest &> ../tmp/forward.log
elif [[ $1 == "front" ]]; then
    pushd ./front &> /dev/null

    echo "booting front hot" | prefix_front

    npm i

    function kill_server() {
        PID=$(lsof -n -i :8999 | grep -oP "(?<=deno)\s+[0-9]+")

        echo "cleanup $PID" | prefix_front
        kill $PID

        if [[ $1 != "keep-alive" ]]; then
            exit 1
        fi
    }

    trap kill_server SIGINT

    export FRONT_PORT=8999
    while true; do
        echo "reboot front" | prefix_front

        deno run \
            --allow-read \
            --allow-write \
            --allow-net \
            --allow-env \
            --allow-run \
            build.ts

        deno run \
            --allow-net \
            --allow-read \
            --allow-env \
            server.ts &

        inotifywait -e modify -e move -e create -e delete -e attrib -r . --exclude=node_modules

        kill_server "keep-alive"
    done
elif [[ $1 == "client" ]]; then
    pushd ./client &> /dev/null

    echo "booting client hot" | prefix_client

    while true; do
        echo "rebuild client" | prefix_client

        wasm-pack build --target web --debug --out-dir ../front/built

        inotifywait -e modify -e move -e create -e delete -e attrib -r . --exclude=target
    done
elif [[ $1 == "renderer" ]]; then
    pushd ./renderer &> /dev/null

    echo "booting renderer hot" | prefix_renderer

    while true; do
        echo "rebuild renderer" | prefix_renderer

        wasm-pack build --target web --debug --out-dir ../front/built

        inotifywait -e modify -e move -e create -e delete -e attrib -r . --exclude=target
    done
elif [[ $1 == "server" ]]; then
    pushd ./server &> /dev/null

    echo "booting server hot" | prefix_server

    function kill_server() {
        PID=$(lsof -n -i :8998 | grep -oP "(?<=server)\s+[0-9]+")

        echo "cleanup $PID" | prefix_server
        kill $PID

        if [[ $1 != "keep-alive" ]]; then
            exit 1
        fi
    }

    trap kill_server SIGINT

    while true; do
        echo "rebuild server" | prefix_server

        cargo build | prefix_server
        
        ./target/debug/server --addr=0.0.0.0:8998 &

        inotifywait -e modify -e move -e create -e delete -e attrib -r . --exclude=target

        kill_server "keep-alive"
    done
else
    echo "invalid arg" | prefix 7 "err"
fi
