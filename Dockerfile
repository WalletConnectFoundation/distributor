################################################################################
#
# Build args
#
################################################################################
ARG                 BASE="rust:1.87-bullseye"
ARG                 RUNTIME="debian:bullseye-slim"
ARG                 VERSION="unknown"
ARG                 SHA="unknown"
ARG                 MAINTAINER="WalletConnect Foundation"
ARG                 PROFILE="release"
ARG                 LOG_LEVEL="debug"
ARG                 WORK_DIR="/app"

################################################################################
#
# Parameters for release builds
#
################################################################################
FROM                ${BASE} AS build-release
ENV                 BUILD_SHARED_ARGS="--profile release"
ENV                 BUILD_PROFILE_DIR="release"

################################################################################
#
# Parameters for debug builds
#
################################################################################
FROM                ${BASE} AS build-debug
ENV                 BUILD_SHARED_ARGS=""
ENV                 BUILD_PROFILE_DIR="debug"

################################################################################
#
# Build the binary
#
################################################################################
FROM                build-${PROFILE} AS build

ARG                 LOG_LEVEL
ARG                 WORK_DIR

RUN                 apt-get update && apt-get install -y --no-install-recommends clang

WORKDIR             ${WORK_DIR}

# Build the local binary
COPY                . .
RUN                 cargo build --bin jupiter-airdrop-api ${BUILD_SHARED_ARGS}

RUN                 ln -s ${WORK_DIR}/target/${BUILD_PROFILE_DIR} ${WORK_DIR}/target/out

################################################################################
#
# Runtime image
#
################################################################################
FROM                ${RUNTIME} AS runtime

ARG                 VERSION
ARG                 SHA
ARG                 MAINTAINER
ARG                 WORK_DIR
ARG                 LOG_LEVEL

LABEL               version=${VERSION}
LABEL               sha=${SHA}
LABEL               maintainer=${MAINTAINER}

RUN                 apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libssl-dev curl \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

WORKDIR             ${WORK_DIR}

COPY --from=build   ${WORK_DIR}/target/out/jupiter-airdrop-api /usr/local/bin/jupiter-airdrop-api 

# Preset the `LOG_LEVEL` env var based on the global log level.
ENV                 LOG_LEVEL="${LOG_LEVEL}"

RUN                 mkdir /jupiter-airdrop-api && chown 7001:7001 /jupiter-airdrop-api

USER                7001:7001
ENTRYPOINT          ["/usr/local/bin/jupiter-airdrop-api"]
