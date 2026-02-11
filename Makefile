# SPDX-FileCopyrightText: 2026 Digg - Agency for Digital Government
#
# SPDX-License-Identifier: EUPL-1.2

.PHONY: all help clean install-targets build-android-libs android aar

.DEFAULT_GOAL := all

ROOT_DIR := $(shell pwd)
LIB_NAME := libopaque_ke_uniffi.so
ANDROID_PROJECT_DIR := $(ROOT_DIR)/android

# Correlate Rust target arch with Android jniLibs arch folder name
TARGET_TO_ANDROID_ARCH := aarch64-linux-android:arm64-v8a armv7-linux-androideabi:armeabi-v7a x86_64-linux-android:x86_64 i686-linux-android:x86

# Define the final locations of the .so files
SO_TARGETS := $(patsubst %:*,$(ANDROID_PROJECT_DIR)/src/main/jniLibs/*/$(LIB_NAME),$(TARGET_TO_ANDROID_ARCH))

KOTLIN_BINDINGS_DIR := $(ANDROID_PROJECT_DIR)/src/main/java
KOTLIN_BINDINGS_FILE := $(KOTLIN_BINDINGS_DIR)/se/digg/opaque_ke_uniffi/opaque_ke_uniffi.kt

AAR_OUTPUT_DIR := $(ROOT_DIR)/build
FINAL_AAR_FILE := $(AAR_OUTPUT_DIR)/opaque_ke_uniffi-release.aar

help:
	@echo "Usage: make [target]"
	@echo " all                - clean, build, and package the AAR"
	@echo " clean              - clean up all build artifacts"
	@echo " install-targets    - install required rust targets for android"
	@echo " build-android-libs - build all android .so files directly into the android/ project"
	@echo " android            - generate .kt bindings directly into the android/ project"
	@echo " aar                - build and package the AAR file"

all: aar

clean:
	@echo "Cleaning..."
	@rm -rf $(AAR_OUTPUT_DIR) $(ANDROID_PROJECT_DIR)/build
	@rm -rf $(ANDROID_PROJECT_DIR)/src/main/jniLibs/*
	@rm -rf $(ANDROID_PROJECT_DIR)/src/main/java/*

# Install required rust targets for android
install-targets:
	@echo "Checking and installing missing Android Rust targets..."
	@$(foreach pair, $(TARGET_TO_ANDROID_ARCH), \
		TARGET_ARCH=$(word 1, $(subst :, ,$(pair))); \
		if ! rustup target list --installed | grep -q $$TARGET_ARCH; then \
			echo "Installing target: $$TARGET_ARCH"; \
			rustup target add $$TARGET_ARCH; \
		fi; \
	)

# Build all the Android libraries
build-android-libs: install-targets
	@echo "Building native libraries..."
	@$(foreach pair, $(TARGET_TO_ANDROID_ARCH), \
		TARGET_ARCH=$(word 1, $(subst :, ,$(pair))); \
		echo "Building for $$TARGET_ARCH..."; \
		cargo ndk -t $$TARGET_ARCH -o $(ANDROID_PROJECT_DIR)/src/main/jniLibs --platform 31 build --release; \
	)

# Generate Kotlin bindings. The .so file is independent of architecture, so we're just using the x86_64 one...
android: build-android-libs
	@echo "Generating Kotlin bindings..."
	@cargo run --bin uniffi-bindgen --release -- generate \
		--library $(ANDROID_PROJECT_DIR)/src/main/jniLibs/x86_64/$(LIB_NAME) \
		--language kotlin \
		--out-dir $(KOTLIN_BINDINGS_DIR)

# Build the AAR. This is the main goal, depending on the artifacts being in place.
aar: clean android
	@echo "--- Packaging AAR ---"
	@echo "1. Building AAR with Gradle..."
	@(cd $(ANDROID_PROJECT_DIR) && ./gradlew --quiet build)

	@echo "2. Copying AAR to build directory..."
	@mkdir -p $(AAR_OUTPUT_DIR)
	@cp $(ANDROID_PROJECT_DIR)/build/outputs/aar/*-release.aar $(FINAL_AAR_FILE)

	@echo "✅ Done! AAR created at $(FINAL_AAR_FILE)"

swift-setup:
    # Version 0.9.0 in git HEAD uses uniffi 0.30.0, which is what we use in this project
	cargo install cargo-swift@0.9.0 -f --git https://github.com/antoniusnaumann/cargo-swift

swift:
	cargo swift package -y --xcframework-name OpaqueKeUniffi
