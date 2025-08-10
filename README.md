For the code to be buildable the following needs to be present:
- on Debian `libssl-dev `
- on NixOS `openssl`

This crate is XDG compliant. The below can be overwritten with environment variables:
- location of logs with `XDG_STATE_HOME`
- location of auth file with `XDG_CONFIG_HOME`
- location of lock fil with `XDG_RUNTIME_DIR`

For authentication, the encrypted gpg file `~/.config/ip_updater/.ip_updater_auth.gpg` (`~/config` can be overwritten with `XDG_CONFIG_HOME` as mentioned) needs to exist. The file itself is should contain data in JSON format with values for 3 keys as below:
- `key` being the api token
- `zone` containing the zone ID
- `record` containing the record ID. This one is tricky but can be obtained using a slightly modified version of the curl command showed [here](https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records). Explicitly, `curl --request GET --url https://api.cloudflare.com/client/v4/zones/<zone id>/dns_records --header 'Bearer <API token>'`. Replace the `zone id` and `API token` with yours.

**Note:** Don't use your global access token. Create a token with a narrow scope. Cloudflare has helpful templates when you go to create the token.

If there are any questions, please drop me an e-mail at cristian.capsuna@gmail.com
