//! The TLS manager is defined in this module

use std::fmt;
use std::fs;
use std::fs::set_permissions;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use log::error;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::thread_rng;
use rcgen::Certificate;
use rcgen::CertificateParams;
use rcgen::KeyPair;
use thiserror::Error;
use tonic::transport::Certificate as TonicCertificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Identity;
use tonic::transport::ServerTlsConfig;
use url::Url;

use crate::api::handler::common::error::ApiError;
use crate::api::handler::leeches::schema::LeechTlsConfig;
use crate::modules::tls::cert::CertificateBuilder;

mod cert;

/// Struct managing all tls related data used in grpc
pub struct TlsManager {
    /// The CA's certificate with its private key
    ca: Certificate,

    /// The keypair of the CA
    ca_key: KeyPair,

    /// The CA as expected by `tonic`
    tonic_ca: TonicCertificate,

    /// The kraken's server cert and private key
    server: Identity,

    /// The randomly generated fake domain for the kraken to be used for sni
    domain: String,
}

impl TlsManager {
    /// Initialise the manager.
    ///
    /// This function takes a directory where the kraken can safely store persistent data.
    /// (This would normally just be `/var/lib/kraken`)
    ///
    /// If this directory contains a `tls` sub-dir, data will be loaded from it.
    /// If it doesn't everything will be generated and written to the newly created `tls` sub-dir.
    pub fn load(var: impl AsRef<Path>) -> Result<Self, TlsManagerError> {
        let base_path = var.as_ref().join("tls");
        let ca_cert_path = base_path.join("ca.crt");
        let ca_key_path = base_path.join("ca.key");
        let server_cert_path = base_path.join("server.crt");
        let server_key_path = base_path.join("server.key");
        let domain_path = base_path.join("domain");

        let domain;
        let ca;
        let ca_key;
        let server;
        let server_key;

        if base_path.exists() {
            // Read from fs
            domain = fs::read_to_string(domain_path)?;
            let ca_cert_pem = fs::read_to_string(ca_cert_path)?;
            let ca_key_pem = fs::read_to_string(ca_key_path)?;
            ca_key = KeyPair::from_pem(&ca_key_pem)?;
            ca = CertificateParams::from_ca_cert_pem(&ca_cert_pem)?.self_signed(&ca_key)?;

            let server_cert_pem = fs::read_to_string(server_cert_path)?;
            let server_key_pem = fs::read_to_string(server_key_path)?;
            server_key = KeyPair::from_pem(&server_key_pem)?;
            server = CertificateParams::from_ca_cert_pem(&server_cert_pem)?.signed_by(
                &server_key,
                &ca,
                &ca_key,
            )?;
        } else {
            // Generate
            let mut bytes = Vec::with_capacity(32);
            bytes.extend(
                Uniform::new_inclusive(b'a', b'z')
                    .sample_iter(&mut thread_rng())
                    .take(32),
            );

            // Attributes on expressions are experimental, so we annotate this way
            #[allow(clippy::expect_used)]
            {
                domain = String::from_utf8(bytes).expect("[a-z]{32} should be a valid utf8 string");
            }
            let ca_params = cert::CA.build()?;
            ca_key = KeyPair::generate()?;
            ca = ca_params.self_signed(&ca_key)?;

            let server_params = cert::Kraken {
                domain: domain.clone(),
            }
            .build()?;
            server_key = KeyPair::generate()?;
            server = server_params.signed_by(&server_key, &ca, &ca_key)?;

            // Write to fs
            fs::create_dir(&base_path)?;
            set_permissions(base_path, Permissions::from_mode(0o700))?;
            fs::write(domain_path, &domain)?;
            fs::write(ca_cert_path, ca.pem())?;
            fs::write(ca_key_path, ca_key.serialize_pem())?;
            fs::write(server_cert_path, server.pem())?;
            fs::write(server_key_path, server_key.serialize_pem())?;
        }

        // Convert to tonic
        let tonic_ca = TonicCertificate::from_pem(ca.pem());
        let server = Identity::from_pem(server.pem(), server_key.serialize_pem());

        Ok(Self {
            ca,
            ca_key,
            tonic_ca,
            server,
            domain,
        })
    }

    /// Generate a new certificate for a leech.
    ///
    /// Also returns everything else the leech needs in order to do tls.
    pub fn gen_leech_cert(&self, url: Url) -> Result<LeechTlsConfig, TlsManagerError> {
        let cert = cert::Leech { url }.build()?;
        let ca_pem = self.ca.pem();
        let key = KeyPair::generate()?;
        let cert_pem = cert.signed_by(&key, &self.ca, &self.ca_key)?;
        Ok(LeechTlsConfig {
            ca: ca_pem,
            cert: cert_pem.pem(),
            key: key.serialize_pem(),
            sni: self.domain.clone(),
        })
    }

    /// Get tonic's tls config to use when connecting to the leech
    pub fn tonic_client(&self) -> ClientTlsConfig {
        ClientTlsConfig::new().ca_certificate(self.tonic_ca.clone())
    }

    /// Get tonic's tls config to use when listening for leeches' connections
    pub fn tonic_server(&self) -> ServerTlsConfig {
        ServerTlsConfig::new().identity(self.server.clone())
    }
}

/// Error produced by [`TlsManager`]
#[derive(Debug, Error)]
pub enum TlsManagerError {
    /// Failed to generate x509 related data
    #[error("{0}")]
    Rcgen(#[from] rcgen::Error),

    /// Failed to access the file system
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl From<TlsManagerError> for ApiError {
    fn from(value: TlsManagerError) -> Self {
        error!("tls manager error in api: {value}");
        Self::InternalServerError
    }
}

impl fmt::Debug for TlsManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsManager").finish_non_exhaustive()
    }
}
