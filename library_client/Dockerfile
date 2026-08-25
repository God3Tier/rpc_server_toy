FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl xz-utils build-essential ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/*

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

COPY library_client ./zig-client
RUN cd zig-client && zig build

COPY mock/ctypes_test.py ./ctypes_test.py

COPY library_client/run.sh /usr/local/bin/run.sh
RUN chmod +x /usr/local/bin/run.sh
ENTRYPOINT ["/usr/local/bin/run.sh"]