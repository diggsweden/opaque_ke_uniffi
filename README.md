# Opaque_ke_uniffi

A Rust library that provides [UniFFI](https://github.com/mozilla/uniffi-rs/) bindings around
the [opaque-ke](https://github.com/facebook/opaque-ke) crate.
The goal of opaque_ke_uniffi is to make [OPAQUE](https://datatracker.ietf.org/doc/rfc9807/) usable
from **Android (Kotlin)** and **Swift (iOS)** via
generated foreign-language bindings.

## Exported Functions

### Client

| Function                        | Description                                                    |
|---------------------------------|----------------------------------------------------------------|
| `client_registration_start`     | Begin registration with the user's password                    |
| `client_registration_finish`    | Complete registration given the server's response              |
| `client_login_start`            | Begin login with the user's password                           |
| `client_login_finish`           | Complete login and derive session key + export key             |

### Server

| Function                       | Description                                                    |
|--------------------------------|----------------------------------------------------------------|
| `server_setup`                 | Generate a new server setup (keypair + OPRF seed)              |
| `server_registration_start`    | Process a client registration request                          |
| `server_registration_finish`   | Finalize registration and produce a storable password file      |
| `server_login_start`           | Process a client credential request                            |
| `server_login_finish`          | Verify the client's credential finalization and derive session key |


## Building

### Android (`.aar`)

#### Using Docker (recommended)
`docker build --output build . `  
Produces a `.aar` in the `build/` directory - no local dependencies required.

#### Using Make
`make aar`  
Requires the Android SDK/NDK, `cargo-ndk`, and the Gradle wrapper in `android/`  
Individual targets are also available. List with:  
`make help`

## iOS

Use make to build a .xcframework file:  
`make swift-setup`  
`make swift`
