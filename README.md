For this code to run you must have installed `libssl-dev`. On Debian this can be done by simply running `sudo apt install libssl-dev`

For authenticataion, a gpg encrypted file needs to exist at the location `/home/ip_updater/.ip_updater_auth.gpg`. This is the home directory of a user called `ip_updater`. The file itself is should contain data in JSON format with values for 3 keys as below:
- `key` being the api token
- `zone` containing the zone ID
- `record` containing the record ID. This one is tricky but can be obtained using a slightly modified version of the curl command showed [here](https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records). Explicitly, `curl --request GET --url https://api.cloudflare.com/client/v4/zones/<zone id>/dns_records --header 'Bearer <API token>'`. Replace the `zone id` and `API token` with yours.

**Note:** Don't use your global access token. Create a token with a narrow scope. Cloudflare has helpful templates when you go to create the token.

If there are any questions, please drop me an e-mail at cristian.capsuna@gmail.com

