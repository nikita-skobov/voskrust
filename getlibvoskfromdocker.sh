#!/usr/bin/env bash

docker build -t voskbuilder .
id=$(docker create voskbuilder)
docker cp $id:/opt/vosk-api/src/copydir/libvosk.a ./
docker rm -v $id
