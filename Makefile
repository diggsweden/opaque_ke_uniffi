# SPDX-FileCopyrightText: 2026 Digg - Agency for Digital Government
#
# SPDX-License-Identifier: EUPL-1.2

.PHONY: all help clean install-targets build-android-libs android aar desktop desktop-libs desktop-jar

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

# Desktop (host-JVM) build: the same bindings call into native code via JNA, which
# also works off-device. JNA locates the library from the classpath using a folder
# named "<os>-<arch>" (its RESOURCE_PREFIX), so we stage the host library there and
# zip it into a jar that the consuming Android project pulls in as a testImplementation.
DESKTOP_BASE_NAME := opaque_ke_uniffi
DESKTOP_STAGE := $(AAR_OUTPUT_DIR)/desktop-jna
DESKTOP_JAR := $(AAR_OUTPUT_DIR)/opaque_ke_uniffi-desktop.jar

help:
	@echo "Usage: make [target]"
	@echo " all                - clean, build, and package the AAR"
	@echo " clean              - clean up all build artifacts"
	@echo " install-targets    - install required rust targets for android"
	@echo " build-android-libs - build all android .so files directly into the android/ project"
	@echo " android            - generate .kt bindings directly into the android/ project"
	@echo " aar                - build and package the AAR file"
	@echo " desktop-libs       - build the host-native library for JVM unit tests (no NDK)"
	@echo " desktop-jar        - package host-native libs into opaque_ke_uniffi-desktop.jar"
	@echo " desktop            - alias for desktop-jar"

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

# Build the host-native library (no Android NDK) and stage it under the JNA
# resource-prefix folder for the current OS/arch. The Kotlin bindings are
# architecture-independent and are reused as-is from the AAR's classes.jar, so
# this target only produces the native binary the host JVM can actually load.
#
# Build natively on each platform you want to test on (e.g. run this on a Mac to
# get the darwin library). To assemble a multi-platform jar, run desktop-libs on
# each OS into a shared $(DESKTOP_STAGE) before running desktop-jar.
desktop-libs:
	@echo "Building host-native library (cargo build --release)..."
	@cargo build --release
	@OS=$$(uname -s); ARCH=$$(uname -m); \
	case "$$OS" in \
		Linux)  OSP=linux;  EXT=so ;; \
		Darwin) OSP=darwin; EXT=dylib ;; \
		*) echo "Unsupported host OS for desktop build: $$OS" >&2; exit 1 ;; \
	esac; \
	case "$$ARCH" in \
		x86_64|amd64)   JARCH=x86-64 ;; \
		arm64|aarch64)  JARCH=aarch64 ;; \
		*) echo "Unsupported host arch for desktop build: $$ARCH" >&2; exit 1 ;; \
	esac; \
	PREFIX=$$OSP-$$JARCH; \
	LIB=lib$(DESKTOP_BASE_NAME).$$EXT; \
	SRC=$(ROOT_DIR)/target/release/$$LIB; \
	if [ ! -f "$$SRC" ]; then echo "Expected $$SRC not found" >&2; exit 1; fi; \
	DEST=$(DESKTOP_STAGE)/$$PREFIX; \
	rm -rf "$$DEST"; mkdir -p "$$DEST"; \
	cp "$$SRC" "$$DEST/$$LIB"; \
	echo "Staged $$LIB -> $$DEST/$$LIB (JNA prefix: $$PREFIX)"

# Package every staged host library into a single jar laid out so JNA finds them
# on the test classpath: /<os>-<arch>/lib<name>.{so,dylib,dll}
desktop-jar: desktop-libs
	@echo "--- Packaging desktop jar ---"
	@mkdir -p $(AAR_OUTPUT_DIR)
	@jar cf $(DESKTOP_JAR) -C $(DESKTOP_STAGE) .
	@echo "✅ Done! Desktop jar created at $(DESKTOP_JAR)"
	@echo "   Contents:"; jar tf $(DESKTOP_JAR) | sed 's/^/     /'

desktop: desktop-jar

swift-setup:
# cargo-swift 0.11.0 pins uniffi_bindgen =0.31.0; Cargo.toml must match exactly
	cargo install cargo-swift -f --git https://github.com/antoniusnaumann/cargo-swift --tag v0.11.0

swift:
	cargo swift package -y
