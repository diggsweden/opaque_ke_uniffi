use opaque_ke::CipherSuite;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RegistrationError {
    #[error("Registration failed: {0}")]
    Generic(String),
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LoginError {
    #[error("Login failed: {0}")]
    Generic(String),
}

#[allow(dead_code)]
pub struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = p256::NistP256;
    type KeyExchange = opaque_ke::TripleDh<p256::NistP256, sha2::Sha256,>;
    type Ksf = opaque_ke::ksf::Identity;
}
