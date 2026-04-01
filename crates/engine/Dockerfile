FROM docker.io/library/rust:1.94-trixie AS build

WORKDIR /usr/src/florca

RUN --mount=type=bind,source=.cargo,target=.cargo \
  --mount=type=bind,source=crates,target=crates \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/usr/src/florca/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
cargo build --release -p florca-engine
cp target/release/florca-engine /usr/local/bin/florca-engine
EOF

# ---

FROM docker.io/library/debian:trixie-slim

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/florca

COPY --from=docker.io/denoland/deno:bin-2.7.9 /deno /usr/local/bin/deno
COPY deno.json deno.lock ./
# Ideally, we would install the dependencies here, but the `deno install` command fails with the following error:
#   Could not find config file for workspace member in 'file:///usr/src/florca/packages/fn/'
# See https://github.com/denoland/deno/issues/28365.
# RUN deno install
COPY packages packages
RUN deno install --entrypoint packages/driver/main.ts

COPY --from=build /usr/local/bin/florca-engine /usr/local/bin/florca-engine
CMD ["florca-engine"]
