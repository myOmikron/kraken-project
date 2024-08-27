//! This modules defines how certs are created i.e. what parameters are set

use std::net::IpAddr;

use rcgen::BasicConstraints;
use rcgen::CertificateParams;
use rcgen::DnType;
use rcgen::ExtendedKeyUsagePurpose;
use rcgen::Ia5String;
use rcgen::IsCa;
use rcgen::KeyUsagePurpose;
use rcgen::SanType;
use url::Host;
use url::Url;

/// [`CertificateBuilder`] which builds the kraken's CA
pub struct CA;
impl CertificateBuilder for CA {
    fn params(self, params: &mut CertificateParams) -> Result<(), rcgen::Error> {
        params
            .distinguished_name
            .push(DnType::CommonName, "kraken CA");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages.push(KeyUsagePurpose::KeyCertSign);

        Ok(())
    }
}

/// [`CertificateBuilder`] which builds the kraken's server certificate
pub struct Kraken {
    /// The randomly generated fake domain for the kraken to be used for sni
    pub domain: String,
}
impl CertificateBuilder for Kraken {
    fn params(self, params: &mut CertificateParams) -> Result<(), rcgen::Error> {
        params
            .distinguished_name
            .push(DnType::CommonName, "kraken cert");
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages.extend([
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ]);
        params
            .extended_key_usages
            .extend([ExtendedKeyUsagePurpose::ServerAuth]);
        params
            .subject_alt_names
            .extend([SanType::DnsName(Ia5String::try_from(self.domain)?)]);
        params.use_authority_key_identifier_extension = true;

        Ok(())
    }
}

/// [`CertificateBuilder`] which builds a leech's server certificate
pub struct Leech {
    /// The uri used to connect to the leech via grpc
    pub url: Url,
}
impl CertificateBuilder for Leech {
    fn params(self, params: &mut CertificateParams) -> Result<(), rcgen::Error> {
        params
            .distinguished_name
            .push(DnType::CommonName, "leech cert");
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages.extend([
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ]);
        params
            .extended_key_usages
            .extend([ExtendedKeyUsagePurpose::ServerAuth]);
        if let Some(host) = self.url.host() {
            params.subject_alt_names.extend([match host {
                Host::Domain(domain) => SanType::DnsName(Ia5String::try_from(domain)?),
                Host::Ipv4(v4) => SanType::IpAddress(IpAddr::V4(v4)),
                Host::Ipv6(v6) => SanType::IpAddress(IpAddr::V6(v6)),
            }]);
        }
        /*params
        .subject_alt_names
        .extend([SanType::URI(self.uri.to_string())]);*/
        params.use_authority_key_identifier_extension = true;

        Ok(())
    }
}

/// Small trait to encapsulate building [`CertificateParams`] from some parameters
pub trait CertificateBuilder: Sized {
    /// Modify an instance of [`CertificateParams::default`] to match the builder's intent.
    fn params(self, params: &mut CertificateParams) -> Result<(), rcgen::Error>;

    /// Consume the builder and create a [`Certificate`]
    fn build(self) -> Result<CertificateParams, rcgen::Error> {
        let mut params = CertificateParams::default();
        self.params(&mut params)?;
        Ok(params)
    }
}
