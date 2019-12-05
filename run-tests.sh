#!/bin/bash

IMAGE=$(docker build -q -f tests.Dockerfile .)
exec docker run --rm -it $IMAGE
