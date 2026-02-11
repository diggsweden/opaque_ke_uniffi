// SPDX-FileCopyrightText: 2026 Digg - Agency for Digital Government
//
// SPDX-License-Identifier: EUPL-1.2

mod client_login;
pub mod client_registration;
pub mod common;
pub mod hash2curve;
pub mod server;
mod server_login;
pub mod server_registration;

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests;
