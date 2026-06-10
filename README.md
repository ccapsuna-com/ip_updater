For the code to be buildable the following needs to be present:
- on Debian `libssl-dev `
- on NixOS `openssl`

This application is designed to run as a Docker container and is configured using Docker secrets and configs.

### Configuration

This service updates the DNS 'A' records for two domains (`ccapsuna.com` and `filotimocreations.com`) using a single shared Cloudflare API token. Each domain has its own Zone ID and Record ID.

The following Docker secrets and configs must be created before deploying the service:

- **Secret**: Your Cloudflare API token (shared by both domains).
- **Config**: The Cloudflare Zone ID for `ccapsuna.com`.
- **Config**: The Record ID of the DNS 'A' record to update for `ccapsuna.com`.
- **Config**: The Cloudflare Zone ID for `filotimocreations.com`.
- **Config**: The Record ID of the DNS 'A' record to update for `filotimocreations.com`.

You can find the `Zone ID` on your Cloudflare dashboard. The `Record ID` can be found by using the Cloudflare API. Use this api call to list all the records on a zone (domain): `curl -s "https://api.cloudflare.com/client/v4/zones/<zone_id>/dns_records?type=A" -H "Authorization: Bearer <your_token>"`

After the docker setup specify the the following environment variables to allow the program to find the required data:
- KEY_PATH. This will be something like `/run/secrets/secret_name`
- CCAPSUNA_ZONE_PATH. The `ccapsuna.com` Zone ID file, something like `/path_config_name`
- CCAPSUNA_RECORD_PATH. The `ccapsuna.com` Record ID file, something like `/record_config_name`
- FILOTIMOCREATIONS_ZONE_PATH. The `filotimocreations.com` Zone ID file
- FILOTIMOCREATIONS_RECORD_PATH. The `filotimocreations.com` Record ID file

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
   echo "CCAPSUNA_ZONE_ID" | docker config create ip_updater_zone -
   echo "CCAPSUNA_RECORD_ID" | docker config create ip_updater_record -
   echo "FILOTIMOCREATIONS_ZONE_ID" | docker config create ip_updater_filotimocreations_zone -
   echo "FILOTIMOCREATIONS_RECORD_ID" | docker config create ip_updater_filotimocreations_record -
   ```

2. Deploy the stack:
   ```bash
   docker stack deploy -c compose.yml ip_updater
   ```
