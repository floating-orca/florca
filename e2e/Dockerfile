FROM docker.io/library/rust:1.90-trixie AS build

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
cargo build --locked --release
cp target/release/florca-deployer /usr/local/bin/florca-deployer
cp target/release/florca-engine /usr/local/bin/florca-engine
cp target/release/florca /usr/local/bin/florca
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
  caddy \
  netcat-traditional \
  postgresql \
  postgresql-client \
  sudo \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/florca

COPY --from=docker.io/denoland/deno:bin-2.5.1 /deno /usr/local/bin/deno
COPY deno.json deno.lock ./
COPY packages packages
RUN deno install --entrypoint packages/driver/main.ts

COPY --from=build /usr/local/bin/florca-deployer /usr/local/bin/florca-deployer
COPY --from=build /usr/local/bin/florca-engine /usr/local/bin/florca-engine
COPY --from=build /usr/local/bin/florca /usr/local/bin/florca

COPY dist/src/.env .env
COPY dist/src/Caddyfile Caddyfile

COPY e2e/container-scripts e2e/container-scripts
RUN e2e/container-scripts/initdb.sh
ENTRYPOINT ["e2e/container-scripts/entrypoint.sh"]

COPY examples/siblings examples/siblings
CMD ["e2e/container-scripts/test.sh"]
