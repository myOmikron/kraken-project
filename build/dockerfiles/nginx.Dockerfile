FROM node:23-bookworm-slim AS build-frontend

WORKDIR /app

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .
COPY ./frontend/ .

RUN --mount=type=cache,target=./node_modules/ \
    <<EOF
set -e
yarn --frozen-lockfile
yarn build
mv ./dist /frontend
EOF


FROM nginx:latest AS final

COPY --from=build-frontend /frontend /usr/share/nginx/html/frontend
COPY ./build/nginx/swagger-initializer.js /usr/share/nginx/html/swagger-ui/