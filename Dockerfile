# =============================================================================
# Stage 1: Install tooling
# =============================================================================
FROM rust:1.86-bookworm AS base

# Android SDK/NDK configuration
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_HOME=${ANDROID_SDK_ROOT}
ENV ANDROID_NDK_VERSION=27.2.12479018
ENV ANDROID_NDK_HOME=${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}

# Install JDK 17 + basic tools
RUN apt-get update && apt-get install -y --no-install-recommends \
        openjdk-17-jdk-headless \
        unzip \
        wget \
    && rm -rf /var/lib/apt/lists/*

# Install Android command-line tools
RUN mkdir -p ${ANDROID_SDK_ROOT}/cmdline-tools && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip \
         -O /tmp/cmdline-tools.zip && \
    unzip -q /tmp/cmdline-tools.zip -d ${ANDROID_SDK_ROOT}/cmdline-tools && \
    mv ${ANDROID_SDK_ROOT}/cmdline-tools/cmdline-tools ${ANDROID_SDK_ROOT}/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

ENV PATH="${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin:${PATH}"

# Accept licenses and install SDK components
RUN yes | sdkmanager --licenses > /dev/null 2>&1 && \
    sdkmanager "platforms;android-34" "ndk;${ANDROID_NDK_VERSION}" "build-tools;34.0.0"

# Install cargo-ndk and Rust cross-compilation targets
RUN cargo install cargo-ndk && \
    rustup target add \
        aarch64-linux-android \
        armv7-linux-androideabi \
        x86_64-linux-android \
        i686-linux-android

WORKDIR /workspace

# =============================================================================
# Stage 2: Cache Rust dependencies (only re-runs when Cargo.toml/lock change)
# =============================================================================
FROM base AS deps

# Copy only dependency manifests first
COPY Cargo.toml Cargo.lock ./
COPY src/bin/uniffi-bindgen.rs src/bin/uniffi-bindgen.rs

# Create a dummy lib.rs so cargo can resolve & fetch dependencies
RUN mkdir -p src && echo "" > src/lib.rs && \
    cargo fetch && \
    rm -rf src

# =============================================================================
# Stage 3: Build the AAR
# =============================================================================
FROM deps AS build

# Copy full source
COPY . .

# 1. Build native .so files for all Android architectures
RUN cargo ndk \
    -t aarch64-linux-android \
    -t armv7-linux-androideabi \
    -t x86_64-linux-android \
    -t i686-linux-android \
    -o android/src/main/jniLibs \
    --platform 31 \
    build --release

# 2. Generate Kotlin bindings
RUN cargo run --bin uniffi-bindgen --release -- generate \
    --library android/src/main/jniLibs/x86_64/libopaque_ke_uniffi.so \
    --language kotlin \
    --out-dir android/src/main/java

# 3. Build AAR with Gradle
RUN cd android && ./gradlew --no-daemon --quiet assembleRelease

# =============================================================================
# Stage 4: Export just the artifact
# =============================================================================
FROM scratch AS artifact
COPY --from=build /workspace/android/build/outputs/aar/*-release.aar /opaque_ke_uniffi-release.aar
