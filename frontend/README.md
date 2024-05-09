# Frontend

## before running the project

### env file

## in root:

rename `.env.example` to `.env`

## in frontend directory:

rename `.env.copy` to `.env` and set `NEXT_PUBLIC_GEO_SERVICE_URL=http://127.0.0.1:8081`

## Running the project

#### in the root directory run:

```
docker compose down --volumes
cargo clean
tools/generate-secrets.sh
docker compose up (for linux,... users)
docker compose -f docker-compose.mac.yml up (for MACOS users)
tools/run-migrations.sh
cargo build

cargo run --bin simulation-manager
cargo run --bin time-simulator
cargo run --bin ui-backend

cargo run --bin energy-supply-and-demand-simulator
cargo run --bin load-flow
cargo run --bin weather-simulator
```

#### if you want to run the sensors:

```
cargo run --bin sensor-data-generator
cargo run --bin sensor-data-ingestor
cargo run --bin sensor-data-transformer
```

#### in the frontend directory run:

```
yarn install
yarn proto
yarn run build
yarn run dev
```

docker might not have a connection with the backend yet, so you should try to refresh.

### problems

#### problems when running `yarn proto`

error: ./build-proto.sh: Bad substitution

there are two possible solutions:

1. install bash
   `sudo apt-get install bash` for linux or `brew install bash` for macos
2. open the build-proto.sh file and comment the following lines out:

```
#! /bin/sh
#! /usr/bin/env bash
```

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
git clean -d -f .

## Project structure

-   `src/app/` pages and layouts
-   `src/components/` reusable components
-   `src/hooks/` custom hooks (mostly utilities)
-   `src/store/` state management using the Context API
-   `src/api/` backend URLs and API calls

## Guidelines

-   write reusable React components
-   use the Flowbite component library where possible
-   do not overuse the React context library (bad for performance due to excessive rendering)
-   stick to the project structure
-   do not hardcode backend URLs
-   `src/store/` should not depend on `src/api/`, `src/app/`, etc.
-   do not commit/push hardcoded URLs in `src/api/urls.ts`
