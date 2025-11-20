//! This module handles the server-side of the OPAQUE login process.

use crate::common::{DefaultCipherSuite, LoginError};
use opaque_ke::{
    CredentialFinalization, CredentialRequest, Identifiers, ServerLogin, ServerLoginParameters,
    ServerRegistration, ServerSetup,
};
use rand::rngs::OsRng;

#[derive(uniffi::Record)]
pub struct ServerLoginStartResult {
    pub credential_response: Vec<u8>,
    pub server_login: Vec<u8>,
}

/// The first step of the OPAQUE login flow on the server side.
///
/// This function takes a previously generated server setup,
/// the user's password file (from registration),
/// a credential request from the client,
/// a credential identifier to look up the user,
/// and optional parameters for context and identifiers.
/// It returns a credential response to be sent to the client,
/// and a server login state to be used in the finish step.
#[uniffi::export]
pub fn server_login_start(
    server_setup: Vec<u8>,
    password_file: Vec<u8>,
    credential_request: Vec<u8>,
    credential_identifier: &[u8],
    context: Option<Vec<u8>>,
    client_identifier: Option<Vec<u8>>,
    server_identifier: Option<Vec<u8>>,
) -> Result<ServerLoginStartResult, LoginError> {
    let mut rng = OsRng;

    let server_setup_obj = ServerSetup::<DefaultCipherSuite>::deserialize(&server_setup)
        .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let password_opt = match ServerRegistration::<DefaultCipherSuite>::deserialize(&password_file) {
        Ok(r) => Some(r),
        Err(_) => None,
    };

    let credential_req = CredentialRequest::<DefaultCipherSuite>::deserialize(&credential_request)
        .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let params = ServerLoginParameters {
        context: context.as_deref(),
        identifiers: Identifiers {
            client: client_identifier.as_deref(),
            server: server_identifier.as_deref(),
        },
    };

    let login_result = ServerLogin::start(
        &mut rng,
        &server_setup_obj,
        password_opt,
        credential_req,
        credential_identifier,
        params,
    )
    .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    Ok(ServerLoginStartResult {
        credential_response: login_result.message.serialize().to_vec(),
        server_login: login_result.state.serialize().to_vec(),
    })
}

/// Completes the OPAQUE login flow on the server side.
///
/// This function takes the server's login state from the start step,
/// the client's credential finalization message,
/// and optional parameters for context and identifiers.
/// It returns the session key on successful authentication.
#[uniffi::export]
pub fn server_login_finish(
    server_login: Vec<u8>,
    credential_finalization: Vec<u8>,
    context: Option<Vec<u8>>,
    client_identifier: Option<Vec<u8>>,
    server_identifier: Option<Vec<u8>>,
) -> Result<Vec<u8>, LoginError> {
    let server_login_obj = ServerLogin::<DefaultCipherSuite>::deserialize(&server_login)
        .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let credential_finalization_obj =
        CredentialFinalization::<DefaultCipherSuite>::deserialize(&credential_finalization)
            .map_err(|e| LoginError::Generic(format!("{:?}", e)))?;

    let params = ServerLoginParameters {
        context: context.as_deref(),
        identifiers: Identifiers {
            client: client_identifier.as_deref(),
            server: server_identifier.as_deref(),
        },
    };

    let result = server_login_obj
        .finish(
            credential_finalization_obj,
            params,
        )
        .map_err(|e| LoginError::Generic(format!("f****{:?}", e)))?;

    Ok(result.session_key.to_vec())
}
