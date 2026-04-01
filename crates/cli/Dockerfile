FROM docker.io/library/rust:1.94-trixie AS build

WORKDIR /usr/src/florca

RUN --mount=type=bind,source=.cargo,target=.cargo \
  --mount=type=bind,source=crates,target=crates \
  --mount=type=bind,source=templates,target=templates \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/usr/src/florca/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
cargo build --release -p florca
cp target/release/florca /usr/local/bin/florca
EOF

# ---

FROM scratch
COPY --from=build /usr/local/bin/florca /
ENTRYPOINT ["/florca"]
