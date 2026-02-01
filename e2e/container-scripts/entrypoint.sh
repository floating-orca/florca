#!/bin/bash

set -Eeuo pipefail

{ 
  caddy start --config Caddyfile

  sudo -u postgres /usr/lib/postgresql/17/bin/pg_ctl -D /var/lib/postgresql/data start

  florca info

  florca-deployer &
  florca-engine &

  while ! nc -z localhost 8000; do
      sleep 1
  done
  while ! nc -z localhost 8001; do
      sleep 1
  done

} >/dev/null 2>&1

if [ "$#" -gt 0 ]; then
  exec "$@"
else
  # otherwise, keep the container running
  tail -f /dev/null
fi
