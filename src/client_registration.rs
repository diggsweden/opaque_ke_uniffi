//! This module handles the client-side of the OPAQUE registration process.

use crate::common::{DefaultCipherSuite, RegistrationError};
use opaque_ke::{ClientRegistration, ClientRegistrationFinishParameters, Identifiers, RegistrationResponse};
use rand::rngs::OsRng;

#[derive(uniffi::Record)]
pub struct ClientRegistrationStartResult {
    pub registration_request: Vec<u8>,
    pub client_registration: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct ClientRegistrationFinishResult {
    pub registration_upload: Vec<u8>,
    pub export_key: Vec<u8>,
}

/// Initiates the OPAQUE registration flow on the client side.
///
/// This function takes the user's password and returns the initial registration message to be sent to the server,
/// and the client's internal registration state.
#[uniffi::export]
pub fn client_registration_start(
    password: Vec<u8>,
) -> Result<ClientRegistrationStartResult, RegistrationError> {
    let mut rng = OsRng;

    let result = ClientRegistration::<DefaultCipherSuite>::start(&mut rng, &password)
        .map_err(|e| RegistrationError::Generic(format!("{:?}", e)))?;

    Ok(ClientRegistrationStartResult {
        registration_request: result.message.serialize().to_vec(),
        client_registration: result.state.serialize().to_vec(),
    })
}

/// Completes the OPAQUE registration flow on the client side.
///
/// This function takes the user's password, the client's registration state,
/// the server's registration response, and optional client and server identifiers.
/// It returns a RegistrationUpload to be sent to the server and the export key.
#[uniffi::export]
pub fn client_registration_finish(
    password: Vec<u8>,
    client_registration: Vec<u8>,
    registration_response: Vec<u8>,
    client_identifier: Option<Vec<u8>>,
    server_identifier: Option<Vec<u8>>,
) -> Result<ClientRegistrationFinishResult, RegistrationError> {
    let mut rng = OsRng;

    let client_reg =
        match ClientRegistration::<DefaultCipherSuite>::deserialize(&client_registration) {
            Ok(r) => r,
            Err(_) => panic!("Failed to deserialize client registration"),
        };

    let reg_response =
        match RegistrationResponse::<DefaultCipherSuite>::deserialize(&registration_response) {
            Ok(r) => r,
            Err(_) => panic!("Failed to deserialize registration response"),
        };

    let params = ClientRegistrationFinishParameters {
        identifiers: Identifiers {
            client: client_identifier.as_deref(),
            server: server_identifier.as_deref(),
        },
        ksf: None,
    };

    let result = client_reg
        .finish(
            &mut rng,
            &password,
            reg_response,
            params,
        )
        .map_err(|e| RegistrationError::Generic(format!("{:?}", e)))?;

    Ok(ClientRegistrationFinishResult {
        registration_upload: result.message.serialize().to_vec(),
        export_key: result.export_key.to_vec(),
    })
}
