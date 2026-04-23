# Self-hosting

This chapter provides instructions for self-hosting _FloatingOrca_ on a [Hetzner Cloud](https://www.hetzner.com/cloud/) server.

Since we don't want Basic Auth credentials and other sensitive information to be sent in plain text to the remote server, we will enable HTTPS.
More specifically, we will use the `sslip.io` service to create a domain name that points to the server's IP address. When configured correctly, Caddy (our reverse proxy) will automatically obtain a TLS certificate for the domain name and serve the services over HTTPS.

## Prerequisites

- A Hetzner Cloud account

## Server-side setup

1. Log in to your Hetzner Cloud account

2. Create an Ubuntu 24.04 Intel/AMD server (CX22, e.g.)

3. Attach a firewall to the server via Hetzner Cloud's web interface and configure it as follows:

   - Allow TCP traffic on port `22`
   - Allow TCP traffic on port `8080`
   - Allow TCP traffic on port `443`

4. Note the server's IP address

5. SSH into the server:

   ```bash
   ssh root@<your-server-ip>
   ```

6. Upgrade the system and reboot:

   ```bash
   apt update && apt upgrade -y && systemctl reboot
   ```

7. SSH into the server again:

   ```bash
   ssh root@<your-server-ip>
   ```

8. Install `curl` and `jq`:

   ```bash
   apt install -y curl jq
   ```

9. Install Docker using their convenience script:

   ```bash
   curl -fsSL https://get.docker.com | sh
   ```

10. Get the release asset:

    ```bash
    export VERSION=<florca-version> # e.g., 0.8.1
    export USERNAME=<your-github-username>
     export PERSONAL_ACCESS_TOKEN=<your-personal-access-token> # GitHub personal access token (classic) with `write:packages` and `delete:packages` scopes

    release_id=$(curl -s -H "Authorization: token ${PERSONAL_ACCESS_TOKEN}" https://api.github.com/repos/floating-orca/florca/releases | jq -r ".[] | select(.tag_name == \"v${VERSION}\") | .id")

    asset_url=$(curl -s -H "Authorization: token ${PERSONAL_ACCESS_TOKEN}" https://api.github.com/repos/floating-orca/florca/releases/${release_id}/assets | jq -r ".[] | select(.name == \"florca-${VERSION}-linux-amd64.tar.gz\") | .url")

    curl -L -H "Authorization: token ${PERSONAL_ACCESS_TOKEN}" -H "Accept: application/octet-stream" "$asset_url" -o "florca-${VERSION}-linux-amd64.tar.gz"
    ```

11. Extract the release asset:

    ```bash
    tar -xzf "florca-${VERSION}-linux-amd64.tar.gz"
    ```

12. Enter the extracted directory:

    ```bash
    cd florca
    ```

13. Pull the Docker images:

    ```bash
    echo "${PERSONAL_ACCESS_TOKEN}" | docker login ghcr.io --username "${USERNAME}" --password-stdin
    docker pull ghcr.io/floating-orca/deployer:${VERSION}
    docker pull ghcr.io/floating-orca/engine:${VERSION}
    ```

14. Copy the environment file and the Caddyfile to the current directory:

    ```bash
    cp dist/src/.env .env
    cp dist/src/Caddyfile Caddyfile
    ```

15. Export the server's IP address as an environment variable:

    ```bash
    export SERVER_IP=<your-server-ip>
    export SERVER_IP_DASHES=$(echo "$SERVER_IP" | sed 's/\./-/g')
    ```

16. Update `.env` and `Caddyfile` to reflect the server's IP address:

    ```bash
    sed -i -E "s|http://deployer.florca.localhost:8080|https://deployer.${SERVER_IP_DASHES}.sslip.io|" .env Caddyfile
    sed -i -E "s|http://engine.florca.localhost:8080|https://engine.${SERVER_IP_DASHES}.sslip.io|" .env Caddyfile
    ```

17. If you want to deploy and run AWS Lambda functions, set the following environment variables in the `.env` file:

    ```bash
    AWS_ACCESS_KEY_ID=<your-aws-access-key-id>
    AWS_SECRET_ACCESS_KEY=<your-aws-secret-access-key>
    AWS_ROLE=<your-aws-role-arn> # e.g., arn:aws:iam::123456789012:role/YourLambdaExecutionRole
    ```

18. Start the services using Docker Compose:

    ```bash
    docker compose up -d
    ```

## Client-side setup

1. On the client (where you want to run the CLI), update the `.env` file to connect to the server:

   ```bash
   sed -i -E "s|http://deployer.florca.localhost:8080|https://deployer.${SERVER_IP_DASHES}.sslip.io|" .env
   sed -i -E "s|http://engine.florca.localhost:8080|https://engine.${SERVER_IP_DASHES}.sslip.io|" .env Caddyfile
   ```

2. Finally, confirm that the CLI is pointing to the server:

   ```bash
   florca info
   ```

_Note that instead of updating `.env` (or `.env.local`), you can also pass the `--env-file` parameter to CLI commands to specify an additional environment file to load (overriding the values of the default environment files)._

## Usage

Let's deploy and run a simple workflow to test the setup:

```bash
florca deploy -w examples/siblings
florca run -d siblings --wait
```

To verify that AWS Lambda functions can communicate with plugin functions, you can deploy and run an example workflow that includes both:

```bash
florca deploy -w examples/tree
florca run -d tree --wait --entry-point processWithDelay --input '{ "onAws": true }'
```

## Custom domain

If you want to use a custom domain instead of the `sslip.io` service, you can set up a domain name and update the `.env` and `Caddyfile` files accordingly.
Make sure to also update the DNS records to point to your server's IP address.
