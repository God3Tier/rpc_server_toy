FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl xz-utils build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Pin the exact 0.16.x release you're developing against so the container
# always matches what you're running locally -- check ziglang.org/download
# for the current 0.16.x tarball name/URL and fill it in here.
ARG ZIG_VERSION=0.16.0
ARG TARGETARCH
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) ZIG_ARCH="x86_64" ;; \
        arm64) ZIG_ARCH="aarch64" ;; \
        *) echo "unsupported arch: ${TARGETARCH}" && exit 1 ;; \
    esac; \
    curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-${ZIG_ARCH}-linux-${ZIG_VERSION}.tar.xz" \
        -o /tmp/zig.tar.xz \
    && mkdir -p /opt/zig \
    && tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 \
    && ln -s /opt/zig/zig /usr/local/bin/zig \
    && rm /tmp/zig.tar.xz

WORKDIR /work

# Your Zig interposition library
COPY library_client ./zig-client
RUN cd zig-client && zig build

ENV LD_LIBRARY_PATH=/work/lib

COPY library_client/run.sh /usr/local/bin/run.sh
RUN chmod +x /usr/local/bin/run.sh
ENTRYPOINT ["/usr/local/bin/run.sh"]
