#!/bin/bash

set -Eeuo pipefail

mkdir -p /var/log/florca

caddy run --config Caddyfile > /var/log/florca/caddy.log 2>&1 &

# shellcheck disable=SC2024
sudo -u postgres /usr/lib/postgresql/17/bin/pg_ctl -D /var/lib/postgresql/data start > /var/log/florca/postgres.log 2>&1

florca-deployer > /var/log/florca/deployer.log 2>&1 &
florca-engine > /var/log/florca/engine.log 2>&1 &

while ! nc -z localhost 8000; do
    sleep 1
done
while ! nc -z localhost 8001; do
    sleep 1
done

if [ "$#" -gt 0 ]; then
  exec "$@"
else
  # otherwise, keep the container running
  tail -f /dev/null
fi
