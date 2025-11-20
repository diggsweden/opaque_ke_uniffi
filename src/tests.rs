use crate::{
    client_login::{client_login_finish, client_login_start},
    client_registration::{client_registration_finish, client_registration_start},
    server::server_setup,
    server_login::{server_login_finish, server_login_start},
    server_registration::{server_registration_finish, server_registration_start},
};

#[test]
fn full_flow_is_ok() {
    let password = b"password".to_vec();
    let client_id = b"client".to_vec();
    let server_id = b"server".to_vec();
    let context = b"context".to_vec();

    let server_setup_bytes = server_setup();

    // Client registration start
    let client_reg_start_result = client_registration_start(password.clone()).unwrap();
    let registration_request = client_reg_start_result.registration_request;
    let client_registration_state = client_reg_start_result.client_registration;

    // Server registration start
    let registration_response =
        server_registration_start(server_setup_bytes.clone(), registration_request, &client_id)
            .unwrap();

    // Client registration finish
    let client_reg_finish_result = client_registration_finish(
        password.clone(),
        client_registration_state,
        registration_response,
        Some(client_id.clone()),
        Some(server_id.clone()),
    )
    .unwrap();
    let registration_upload = client_reg_finish_result.registration_upload;

    // Server registration finish
    let password_file = server_registration_finish(registration_upload).unwrap();
    assert!(
        !password_file.is_empty(),
        "Password file should not be empty"
    );

    // Client login start
    let client_login_start_result = client_login_start(password.clone()).unwrap();
    let credential_request = client_login_start_result.credential_request;
    let client_login_state = client_login_start_result.client_registration;

    // Server login start
    let server_login_start_result = server_login_start(
        server_setup_bytes,
        password_file,
        credential_request,
        &client_id,
        Some(context.clone()),
        Some(client_id.clone()),
        Some(server_id.clone()),
    )
    .unwrap();
    let credential_response = server_login_start_result.credential_response;
    let server_login_state = server_login_start_result.server_login;

    // Client login finish
    let client_login_finish_result = client_login_finish(
        credential_response,
        client_login_state,
        password,
        context.clone(),
        client_id.clone(),
        server_id.clone(),
    )
    .unwrap();
    let credential_finalization = client_login_finish_result.credential_finalization;
    let client_session_key = client_login_finish_result.session_key;

    // Server-side login finish
    let server_session_key = server_login_finish(
        server_login_state,
        credential_finalization,
        Some(context),
        Some(client_id),
        Some(server_id),
    )
    .unwrap();

    assert!(
        !client_session_key.is_empty(),
        "Client session key should not be empty"
    );
    assert!(
        !server_session_key.is_empty(),
        "Server session key should not be empty"
    );
    assert_eq!(
        client_session_key, server_session_key,
        "Session keys should match"
    );
    assert_eq!(
        client_reg_finish_result.export_key, client_login_finish_result.export_key,
        "Export keys should match"
    )
}

#[test]
fn wrong_password_is_err() {
    let correct_password = b"password".to_vec();
    let wrong_password = b"wrong_password".to_vec();
    let client_id = b"client".to_vec();
    let server_id = b"server".to_vec();
    let context = b"context".to_vec();

    let server_setup_bytes = server_setup();

    // Client registration start
    let client_reg_start_result = client_registration_start(correct_password.clone()).unwrap();
    let registration_request = client_reg_start_result.registration_request;
    let client_registration_state = client_reg_start_result.client_registration;

    // Server registration start
    let registration_response =
        server_registration_start(server_setup_bytes.clone(), registration_request, &client_id)
            .unwrap();

    // Client registration finish
    let client_reg_finish_result = client_registration_finish(
        correct_password.clone(),
        client_registration_state,
        registration_response,
        Some(client_id.clone()),
        Some(server_id.clone()),
    )
    .unwrap();
    let registration_upload = client_reg_finish_result.registration_upload;

    // Server registration finish
    let password_file = server_registration_finish(registration_upload).unwrap();

    // Client login start
    let client_login_start_result = client_login_start(correct_password).unwrap();
    let credential_request = client_login_start_result.credential_request;
    let client_login_state = client_login_start_result.client_registration;

    // Server login start
    let server_login_start_result = server_login_start(
        server_setup_bytes,
        password_file,
        credential_request,
        &client_id,
        Some(context.clone()),
        Some(client_id.clone()),
        Some(server_id.clone()),
    )
    .unwrap();
    let credential_response = server_login_start_result.credential_response;

    // Client login finish
    let client_login_finish_result = client_login_finish(
        credential_response,
        client_login_state,
        wrong_password,
        context,
        client_id,
        server_id,
    );

    assert!(client_login_finish_result.is_err());
}

#[test]
fn login_with_no_password_file_is_err() {
    let password = b"password".to_vec();
    let client_id = b"client".to_vec();
    let server_id = b"server".to_vec();
    let context = b"context".to_vec();

    let server_setup_bytes = server_setup();

    // Client login start
    let client_login_start_result = client_login_start(password.clone()).unwrap();
    let credential_request = client_login_start_result.credential_request;
    let client_login_state = client_login_start_result.client_registration;

    // Server login start
    let server_login_start_result = server_login_start(
        server_setup_bytes,
        Vec::new(), // Empty password file
        credential_request,
        &client_id,
        Some(context.clone()),
        Some(client_id.clone()),
        Some(server_id.clone()),
    )
    .unwrap();
    let credential_response = server_login_start_result.credential_response;

    // Client login finish
    let client_login_finish_result = client_login_finish(
        credential_response,
        client_login_state,
        password,
        context,
        client_id,
        server_id,
    );

    assert!(client_login_finish_result.is_err());
}
