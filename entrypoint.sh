#!/bin/bash

# exit when any command fails
set -e

# create a tun device
mkdir -p /dev/net
mknod /dev/net/tun c 10 200
chmod 600 /dev/net/tun

# if reg.json not exists, write some auth info
if [ ! -f /var/lib/cloudflare-warp/reg.json ]; then
  {
    echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
    echo "<dict>"
    echo "    <key>organization</key>"
    echo "    <string>$WARP_ORGANIZATION</string>"
    echo "    <key>auth_client_id</key>"
    echo "    <string>$WARP_AUTH_CLIENT_ID</string>"
    echo "    <key>auth_client_secret</key>"
    echo "    <string>$WARP_AUTH_CLIENT_SECRET</string>"
    echo "</dict>"
  } >/var/lib/cloudflare-warp/mdm.xml
else
  echo "Warp client already registered, skip registration"
fi

# start the proxy
warp-guard \
  --listen-port "$LISTEN_PORT" \
  --healthcheck-start-period "$HEALTHCHECK_START_PERIOD" \
  --healthcheck-interval "$HEALTHCHECK_INTERVAL" \
  --healthcheck-retries "$HEALTHCHECK_RETRIES" \
  --healthcheck-timeout "$HEALTHCHECK_TIMEOUT" \
  --warp-cli-delay "$WARP_CLI_DELAY" \
  --display-gost-logs "$DISPLAY_GOST_LOGS"
