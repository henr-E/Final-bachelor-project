# Frontend

## Running the project
1. generate the proto files (the files should appear in the frontend/src/proto directory)
```
cd frontend
./build-proto.sh
```
2. run envoy service (root directory): `envoy -c envoy.yaml`
3. start the backend:
`cargo run --bin ui-backend`
4. start the frontend:
```
cd frontend
npm install
npm run dev
```
5. navigate to the http://localhost:3000
6. The envoy.yaml file is configured such that the admin access can be found through http://127.0.0.1:9901/
7. to check if envoy is running correctly run `lsof -i :9901`

## Getting Started
1. rename `.env.copy` to `.env` 
2. fill in `.env` with backend URLs
3. load `.env` file into terminal session (eg. `export $(cat .env | xargs)`)
4. `npm i`
5. `npm run dev`
6. navigate to http://localhost:3000

Fill in your backend URLs in `src/api/urls.ts` to avoid repeating step 3 (do not commit changes to this file).

### Installing Envoy

A proxy server is needed to use gRPC within a browser: https://www.envoyproxy.io/

## Learning

1. https://react.dev/learn 
2. https://www.typescriptlang.org/docs/
3. https://nextjs.org/
4. https://react-leaflet.js.org/
5. https://www.flowbite-react.com/
6. https://react.dev/learn/passing-data-deeply-with-context
7. https://react.dev/reference/react/hooks
8. https://nextjs.org/docs/app/building-your-application/routing/defining-routes

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
