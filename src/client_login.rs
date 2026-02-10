// SPDX-FileCopyrightText: 2026 Digg - Agency for Digital Government
//
// SPDX-License-Identifier: EUPL-1.2

//! This module handles the client-side of the OPAQUE login process.

use crate::common::{DefaultCipherSuite, LoginError};
use opaque_ke::{ClientLogin, ClientLoginFinishParameters, Identifiers};
use rand::rngs::OsRng;

#[derive(uniffi::Record)]
pub struct ClientLoginStartResult {
    pub credential_request: Vec<u8>,
    pub client_registration: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct ClientLoginFinishResult {
    pub credential_finalization: Vec<u8>,
    pub session_key: Vec<u8>,
    pub export_key: Vec<u8>,
}

/// Initiates the OPAQUE login flow on the client side.
///
/// This function takes the user's password and returns the initial credential request
/// to be sent to the server, as well as the client's internal login state.
#[uniffi::export]
pub fn client_login_start(password: Vec<u8>) -> Result<ClientLoginStartResult, LoginError> {
    let mut rng = OsRng;

    let result = ClientLogin::<DefaultCipherSuite>::start(&mut rng, &password)
        .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    Ok(ClientLoginStartResult {
        credential_request: result.message.serialize().to_vec(),
        client_registration: result.state.serialize().to_vec(),
    })
}

/// Completes the OPAQUE login flow and derives the session key on the client side.
///
/// # Parameters
/// - `credential_response`: The server's credential response message.
/// - `client_registration`: The client's internal login state from `client_login_start`.
/// - `password`: The user's password.
/// - `context`: Application-specific context for the session.
/// - `client_identifier`: The client's identity.
/// - `server_identifier`: The server's identity.
#[uniffi::export]
pub fn client_login_finish(
    credential_response: Vec<u8>,
    client_registration: Vec<u8>,
    password: Vec<u8>,
    context: Vec<u8>,
    client_identifier: Vec<u8>,
    server_identifier: Vec<u8>,
) -> Result<ClientLoginFinishResult, LoginError> {
    let mut rng = OsRng;

    let cred_resp =
        opaque_ke::CredentialResponse::<DefaultCipherSuite>::deserialize(&credential_response)
            .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let login = ClientLogin::<DefaultCipherSuite>::deserialize(&client_registration)
        .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let params = ClientLoginFinishParameters {
        context: Some(&context),
        identifiers: Identifiers {
            client: Some(&client_identifier),
            server: Some(&server_identifier),
        },
        ksf: None,  // Use default KSF from CipherSuite
    };

    let finish_res = match login.finish(
        &mut rng,
        &password,
        cred_resp,
        params,
    ) {
        Ok(res) => {
            res
        }
        Err(e) => {
            return Err(LoginError::Generic(format!("{:?}", e)));
        }
    };


    Ok(ClientLoginFinishResult {
        credential_finalization: finish_res.message.serialize().to_vec(),
        session_key: finish_res.session_key.to_vec(),
        export_key: finish_res.export_key.to_vec(),
    })
}
