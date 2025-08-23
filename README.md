For the code to be buildable the following needs to be present:
- on Debian `libssl-dev `
- on NixOS `openssl`

This application is designed to run as a Docker container and is configured using Docker secrets and configs.

### Configuration

The following Docker secrets and configs must be created before deploying the service:

- **Secret: `ip_updater_key`**: Your Cloudflare API token.
- **Config: `ip_updater_zone`**: The Cloudflare Zone ID for your domain.
- **Config: `ip_updater_record`**: The Record ID of the DNS 'A' record you want to update.

You can find the `Zone ID` on your Cloudflare dashboard. The `Record ID` can be found by using the Cloudflare API.

### Environment Variables

You can also configure the application using environment variables:

- **`LOCK_FILE_DIRECTORY`**: This needs to be set and it might not be in a container environment
- **`IP_UPDATER_INTERVAL_MINUTES`**: The interval in minutes to check for an IP address change. Defaults to `10`.
- **`LOG_LEVEL`**: Sets the logging verbosity. Defaults to `3` (Info).
  - `0`: Off, `1`: Error, `2`: Warn, `3`: Info, `4`: Debug, `5`: Trace

**Note:** Don't use your global access token. Create a token with a narrow scope. Cloudflare has helpful templates when you go to create the token.

### Deployment

This service is intended to be deployed to a Docker Swarm cluster.

1. Create the necessary secrets and configs:
   ```bash
   echo "YOUR_API_TOKEN" | docker secret create ip_updater_key -
   echo "YOUR_ZONE_ID" | docker config create ip_updater_zone -
   echo "YOUR_RECORD_ID" | docker config create ip_updater_record -
   ```

2. Deploy the stack:
   ```bash
   docker stack deploy -c compose.yml ip_updater
   ```
