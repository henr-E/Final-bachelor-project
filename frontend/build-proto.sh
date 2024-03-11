#! /usr/bin/env bash

# Directory where the proto files are located
PROTO_DIR="../proto"

# Create directory if it does not exist
mkdir -p "$PROTO_DIR"

#folder must exist for protoc
mkdir -p "./src/proto"

# Generate typescript code for all proto files except the excluded one
find "$PROTO_DIR" -name "*.proto" | while read -r PROTO_FILE; do
    if [[ "$PROTO_FILE" != ../proto/simulation/*.proto ]]; then
        protoc \
            --plugin=./node_modules/.bin/protoc-gen-ts_proto \
            -I "$PROTO_DIR" \
            --ts_proto_out=./src/proto \
            --ts_proto_opt=env=browser,outputServices=nice-grpc,outputServices=generic-definitions \
            "$PROTO_FILE"
    fi
done

