mod cert;

use std::fs::{set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fmt, fs, io};

use actix_web::web::Data;
use log::error;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rcgen::{Certificate, CertificateParams, KeyPair, RcgenError};
use serde::Serialize;
use thiserror::Error;
use tonic::transport::{
    Certificate as TonicCertificate, ClientTlsConfig, Identity, ServerTlsConfig,
};
use url::Url;
use utoipa::ToSchema;

use crate::api::handler::ApiError;
use crate::modules::tls::cert::CertificateBuilder;

/// Struct managing all tls related data used in grpc
pub struct TlsManager {
    /// The CA's certificate with its private key
    ca: Certificate,

    /// The CA as expected by `tonic`
    tonic_ca: TonicCertificate,

    /// The kraken's server cert and private key
    server: Identity,

    /// The randomly generated fake domain for the kraken to be used for sni
    domain: String,
}

/// The tls related part of a leech's config
#[derive(Debug, ToSchema, Serialize)]
pub struct LeechTlsConfig {
    /// PEM encoded CA managed by kraken
    pub ca: String,

    /// PEM encoded certificate
    pub cert: String,

    /// PEM encoded private key for the certificate
    pub key: String,

    /// The randomly generated fake domain for the kraken to be used for sni
    pub sni: String,
}

impl TlsManager {
    /// Initialise the manager.
    ///
    /// This function takes a directory where the kraken can safely store persistent data.
    /// (This would normally just be `/var/lib/kraken`)
    ///
    /// If this directory contains a `tls` sub-dir, data will be loaded from it.
    /// If it doesn't everything will be generated and written to the newly created `tls` sub-dir.
    pub fn load(var: impl AsRef<Path>) -> Result<Data<Self>, TlsManagerError> {
        let base_path = var.as_ref().join("tls");
        let ca_cert_path = base_path.join("ca.crt");
        let ca_key_path = base_path.join("ca.key");
        let server_cert_path = base_path.join("server.crt");
        let server_key_path = base_path.join("server.key");
        let domain_path = base_path.join("domain");

        let domain;
        let ca;
        let server;

        if base_path.exists() {
            // Read from fs
            domain = fs::read_to_string(domain_path)?;
            let ca_cert_pem = fs::read_to_string(ca_cert_path)?;
            let ca_key_pem = fs::read_to_string(ca_key_path)?;
            ca = Certificate::from_params(CertificateParams::from_ca_cert_pem(
                &ca_cert_pem,
                KeyPair::from_pem(&ca_key_pem)?,
            )?)?;
            let server_cert_pem = fs::read_to_string(server_cert_path)?;
            let server_key_pem = fs::read_to_string(server_key_path)?;
            server = Certificate::from_params(CertificateParams::from_ca_cert_pem(
                &server_cert_pem,
                KeyPair::from_pem(&server_key_pem)?,
            )?)?;
        } else {
            // Generate
            let mut bytes = Vec::with_capacity(32);
            bytes.extend(
                Uniform::new_inclusive(b'a', b'z')
                    .sample_iter(&mut thread_rng())
                    .take(32),
            );
            domain = String::from_utf8(bytes).expect("[a-z]{32} should be a valid utf8 string");
            ca = cert::CA.build()?;
            server = cert::Kraken {
                domain: domain.clone(),
            }
            .build()?;

            // Write to fs
            fs::create_dir(&base_path)?;
            set_permissions(base_path, Permissions::from_mode(0o700))?;
            fs::write(domain_path, &domain)?;
            fs::write(ca_cert_path, ca.serialize_pem()?)?;
            fs::write(ca_key_path, ca.serialize_private_key_pem())?;
            fs::write(server_cert_path, server.serialize_pem_with_signer(&ca)?)?;
            fs::write(server_key_path, server.serialize_private_key_pem())?;
        }

        // Convert to tonic
        let tonic_ca = TonicCertificate::from_pem(ca.serialize_pem()?);
        let server = Identity::from_pem(
            server.serialize_pem_with_signer(&ca)?,
            server.serialize_private_key_pem(),
        );

        Ok(Data::new(Self {
            ca,
            tonic_ca,
            server,
            domain,
        }))
    }

    /// Generate a new certificate for a leech.
    ///
    /// Also returns everything else the leech needs in order to do tls.
    pub fn gen_leech_cert(&self, url: Url) -> Result<LeechTlsConfig, TlsManagerError> {
        let cert = cert::Leech { url }.build()?;
        let ca_pem = self.ca.serialize_pem()?;
        let cert_pem = cert.serialize_pem_with_signer(&self.ca)?;
        let key_pem = cert.serialize_private_key_pem();
        Ok(LeechTlsConfig {
            ca: ca_pem,
            cert: cert_pem,
            key: key_pem,
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
        //.client_auth_optional(false)
        //.client_ca_root(self.tonic_ca.clone())
    }
}

/// Error produced by [`TlsManager`]
#[derive(Debug, Error)]
pub enum TlsManagerError {
    /// Failed to generate x509 related data
    #[error("{0}")]
    Rcgen(#[from] RcgenError),

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
