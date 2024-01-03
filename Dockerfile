FROM debian:bookworm

# https://github.com/ginuerzh/gost/releases
ARG GOST_VER=2.11.5
ARG GUARD_VER=0.1.0

# install dependencies
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y curl gnupg lsb-release && \
    curl -fsSL https://pkg.cloudflareclient.com/pubkey.gpg | gpg --yes --dearmor --output /usr/share/keyrings/cloudflare-warp-archive-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/cloudflare-warp-archive-keyring.gpg] https://pkg.cloudflareclient.com/ $(lsb_release -cs) main" | tee /etc/apt/sources.list.d/cloudflare-client.list && \
    apt-get update && \
    apt-get install -y cloudflare-warp && \
    apt-get clean && \
    apt-get autoremove -y && \
    curl -LO https://github.com/ginuerzh/gost/releases/download/v$GOST_VER/gost-linux-amd64-$GOST_VER.gz && \
    gunzip gost-linux-amd64-$GOST_VER.gz && \
    mv gost-linux-amd64-$GOST_VER /usr/bin/gost && \
    chmod +x /usr/bin/gost && \
    curl -L https://github.com/docker/compose/releases/download/v${GUARD_VER}/warp-guard-linux-x86_64 -o /usr/bin/warp-guard && \
    chmod +x /usr/bin/warp-guard

# Accept Cloudflare WARP TOS
RUN mkdir -p /root/.local/share/warp && \
    echo -n 'yes' > /root/.local/share/warp/accepted-tos.txt

COPY --chmod=755 entrypoint.sh /entrypoint.sh

# Gost
ENV LISTEN_PORT=1080
ENV DISPLAY_GOST_LOGS=0

# Zero Trust
ENV WARP_ORGANIZATION=""
ENV WARP_AUTH_CLIENT_ID=""
ENV WARP_AUTH_CLIENT_SECRET=""
ENV WARP_CLI_DELAY=3

# Healthcheck
ENV HEALTHCHECK_START_PERIOD=15
ENV HEALTHCHECK_INTERVAL=10
ENV HEALTHCHECK_RETRIES=3
ENV HEALTHCHECK_TIMEOUT=10

ENTRYPOINT ["/entrypoint.sh"]
