#!/bin/bash

docker build . \
    --target=runtime \
    --tag=woods:latest
