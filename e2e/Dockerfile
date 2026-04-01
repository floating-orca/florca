FROM docker.io/library/rust:1.94-trixie AS build

WORKDIR /usr/src/florca

ARG CARGO_PROFILE=dev

RUN --mount=type=bind,source=.cargo,target=.cargo \
  --mount=type=bind,source=crates,target=crates \
  --mount=type=bind,source=templates,target=templates \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/usr/src/florca/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
case "${CARGO_PROFILE}" in
  dev)
    TARGET_DIR_NAME=debug
    CARGO_BUILD_ARGS=""
    ;;
  release)
    TARGET_DIR_NAME=release
    CARGO_BUILD_ARGS="--release"
    ;;
  *)
    echo "Invalid CARGO_PROFILE: ${CARGO_PROFILE}. Expected 'dev' or 'release'." >&2
    exit 1
    ;;
esac
cargo build ${CARGO_BUILD_ARGS}
cp target/${TARGET_DIR_NAME}/florca-deployer /usr/local/bin/florca-deployer
cp target/${TARGET_DIR_NAME}/florca-engine /usr/local/bin/florca-engine
cp target/${TARGET_DIR_NAME}/florca /usr/local/bin/florca
EOF

# ---

FROM docker.io/library/debian:trixie-slim

RUN apt-get update && apt-get install -y \
  ca-certificates \
  debian-keyring \
  debian-archive-keyring \
  apt-transport-https \
  curl \
  && curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg \
  && curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
RUN apt-get update && apt-get install -y \
  bats \
  bats-support \
  bats-assert \
  caddy \
  jq \
  netcat-traditional \
  postgresql \
  postgresql-client \
  sudo \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/florca

COPY e2e/container-scripts e2e/container-scripts
RUN e2e/container-scripts/initdb.sh

COPY --from=docker.io/denoland/deno:bin-2.7.9 /deno /usr/local/bin/deno
COPY deno.json deno.lock ./
COPY packages packages
RUN deno install --entrypoint packages/driver/main.ts

COPY --from=build /usr/local/bin/florca-deployer /usr/local/bin/florca-deployer
COPY --from=build /usr/local/bin/florca-engine /usr/local/bin/florca-engine
COPY --from=build /usr/local/bin/florca /usr/local/bin/florca

COPY dist/src/.env .env
COPY dist/src/Caddyfile Caddyfile

ENTRYPOINT ["e2e/container-scripts/entrypoint.sh"]
