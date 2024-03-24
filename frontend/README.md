# Frontend
## before running the project
### install envoy
##### linux:
   https://www.envoyproxy.io/docs/envoy/latest/start/install#install-envoy-on-ubuntu-linux
##### mac: `brew install envoy`

### env file
rename `.env.example` to `.env`

## Running the project
#### in the root directory run:
```
docker compose down --volumes
cargo clean
tools/generate-secrets.sh
docker compose up
tools/run-migrations.sh
cargo build

cargo run --bin energy-simulator
cargo run --bin simulation-manager
cargo run --bin ui-backend
```
#### in the frontend directory run:
generate the proto files (the files should appear in the frontend/src/proto directory) `yarn proto`

install dependencies: `yarn install`

check if everything is correct: `yarn run build`

deploy: `yarn run dev`

#### run envoy for the backend connection WITHOUT bidirectional streams
navigate to the http://localhost:3000

The envoy.yaml file is configured such that the admin access can be found through http://127.0.0.1:9901/

to check if envoy is running correctly run `lsof -i :9901`

To run envoy without  bidirectional streams ```envoy -c docker/envoy.yaml```

#### run envoy for the backend connection WITH bidirectional streams

To use a stream via web-grpc a tsl certicate is needed (to test this localy), 
this can be genereded with following command:

```openssl req -x509 -out localhost.crt -keyout localhost.key   -newkey rsa:2048 -nodes -sha256```
When creating the cert and key, fill in by ```Common Name (e.g. server FQDN or YOUR name)```: ```localhost```

use the ```envoy -c envoy_bidirectional_stream.yaml``` file to start envoy

When testing, your browser will not trust this certificate. you can trust this in by goint to a backend server link (e.g: https://127.0.0.1:8081/twin.TwinService/getAllTwins). this will show a warning, click on trust certificate.
The stream works only in following browsers: https://caniuse.com/mdn-api_request_request_request_body_readablestream


### problems

#### docker problems:
if you get following error:
Error response from daemon: Ports are not available: exposing port TCP 0.0.0.0:5432 -> 0.0.0.0:0: listen tcp 0.0.0.0:5432: bind: address already in use

check if any process is running on this port:
`sudo lsof -i :5432`

to stop the process:
`sudo kill -9 <PID>`


#### github
if you get following error:
error: The following untracked working tree files would be overwritten by checkout

you can delete all changes by using:
git clean  -d  -f .

## Project structure

- `src/app/` pages and layouts
- `src/components/` reusable components
- `src/hooks/` custom hooks (mostly utilities)
- `src/store/` state management using the Context API
- `src/api/` backend URLs and API calls

## Guidelines

- write reusable React components
- use the Flowbite component library where possible
- do not overuse the React context library (bad for performance due to excessive rendering)
- stick to the project structure
- do not hardcode backend URLs
- `src/store/` should not depend on `src/api/`, `src/app/`, etc.
- do not commit/push hardcoded URLs in `src/api/urls.ts`
