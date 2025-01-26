FROM rust:1.84-alpine AS build

# Install trunk and compiler-toolchain for wasm
RUN apk add musl-dev
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

# Copy source into container
WORKDIR /app
COPY . .

# Build GUI using trunk
WORKDIR /app/gui
RUN trunk build --release


# The container that will be deployed
FROM nginx:stable-alpine AS deployment

COPY --from=build /app/gui/dist /usr/share/nginx/html
