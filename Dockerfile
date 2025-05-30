# Copyright (c) 2023 - 2025 Chair for Design Automation, TUM
# Copyright (c) 2025 Munich Quantum Software Company GmbH
# All rights reserved.
#
# SPDX-License-Identifier: MIT
#
# Licensed under the MIT License

FROM rust:1.84-alpine AS build

# Install trunk and compiler-toolchain for wasm
RUN apk add bash curl musl-dev
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall trunk

# Copy source into container
WORKDIR /app
COPY . .

# Build GUI using trunk
WORKDIR /app/gui
RUN trunk build --release


# The container that will be deployed
FROM nginx:stable-alpine AS deployment

COPY --from=build /app/gui/dist /usr/share/nginx/html
