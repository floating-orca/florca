# Upgrade

<div class="warning">

Note that following the instructions below will remove the database and thus all information regarding deployed workflows and workflow runs.
You will need to redeploy your workflows after the upgrade.

</div>

1. In the `florca` directory (where the `compose.yaml` file is located), run the following command to stop and remove the existing containers:

   ```bash
   docker compose down
   ```

2. Temporarily move your workflow files outside the `florca` directory. E.g.:

   ```bash
   mv workflows ../
   ```

3. Back up your configuration files. E.g.:

    ```bash
    mv .env* Caddyfile ../
    ```

4. Remove the `florca` directory.

5. Start from fresh by following the instructions in the [Getting started](./getting-started.md) chapter.

6. Reapply any configuration changes you made previously.

7. Move your workflow files back into the (new) `florca` directory.

8. If desired, redeploy your workflows using `florca deploy`.
