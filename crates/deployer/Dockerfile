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
cargo build --release -p florca-deployer
cp target/release/florca-deployer /usr/local/bin/florca-deployer
EOF

# ---

FROM docker.io/library/debian:trixie-slim

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/bin/florca-deployer /usr/local/bin/florca-deployer

CMD ["florca-deployer"]
