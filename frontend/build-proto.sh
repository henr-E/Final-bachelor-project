#! /usr/bin/env bash

# Directory where the proto files are located
PROTO_DIR="../proto"

# Create directory if it does not exist
mkdir -p "$PROTO_DIR"

#folder must exist for protoc
mkdir -p "./src/proto"


for f in $(find $PROTO_DIR -type d); do
    # Generate typescript code using
    echo $f
    #folder must exist for protoc
    mkdir -p "./src${f:2}"
    protoc \
        --plugin=./node_modules/.bin/protoc-gen-ts_proto \
        -I "$f" \
        --ts_proto_out="./src${f:2}" \
        --ts_proto_opt=env=browser,outputServices=nice-grpc,outputServices=generic-definitions,esModuleInterop=true,useExactTypes=false \
        "$f"/*.proto
done
