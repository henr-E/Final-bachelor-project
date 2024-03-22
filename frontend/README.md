# Frontend
## before running the project
### install envoy
##### linux:
   https://www.envoyproxy.io/docs/envoy/latest/start/install#install-envoy-on-ubuntu-linux
##### mac
    brew install envoy

### env file
rename `.env.example` to `.env`

#### github
if you get following error:
error: The following untracked working tree files would be overwritten by checkout

you can delete all changes by using:
git clean  -d  -f .




## Running the project
1. generate the proto files (the files should appear in the frontend/src/proto directory)
#### in the root directory run:
```
docker compose down --volumes
tools/generate-secrets.sh
docker compose up
tools/run-migrations.sh
cargo clean
cargo build

cargo run --bin energy-simulator
cargo run --bin simulation-manager
cargo run --bin ui-backend
```
#### in the frontend directory run:
```
yarn proto
yarn install
yarn run build
yarn run dev
```

navigate to the http://localhost:3000

The envoy.yaml file is configured such that the admin access can be found through http://127.0.0.1:9901/

to check if envoy is running correctly run `lsof -i :9901`

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
