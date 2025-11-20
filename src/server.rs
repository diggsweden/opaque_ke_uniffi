use crate::common::DefaultCipherSuite;
use opaque_ke::ServerSetup;
use rand::rngs::OsRng;

#[uniffi::export]
pub fn server_setup() -> Vec<u8> {
    let mut rng = OsRng;

    let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);
    server_setup.serialize().to_vec()
}
