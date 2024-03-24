#! /bin/sh
#! /usr/bin/env bash

# Directory where the proto files are located
PROTO_DIR=${1:-"../proto"}

#folder must exist for protoc
mkdir -p "./src/proto"

PROTO_FILES=$(cd $PROTO_DIR && find . -type d)

for f in $PROTO_FILES; do
    # Generate typescript code using
    echo "Generating: $f"
    #folder must exist for protoc
    mkdir -p "./src/proto/${f:2}"
    protoc \
        --plugin=./node_modules/.bin/protoc-gen-ts_proto \
        -I "$PROTO_DIR/$f" \
        --ts_proto_out="./src/proto/${f:2}" \
        --ts_proto_opt=env=browser,outputServices=nice-grpc,outputServices=generic-definitions,esModuleInterop=true,useExactTypes=false \
        "$PROTO_DIR/$f"/*.proto
done
