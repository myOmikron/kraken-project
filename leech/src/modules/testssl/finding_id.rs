//! Parse and group the finding ids reported by `testssl`

use std::num::NonZeroUsize;
use std::str::FromStr;

/// Enum representing the finding ids `testssl` uses in its json output in a structured way
pub enum FindingId {
    /// Issues with your testssl installation
    ScanIssues(ScanIssues),

    /// Ids which are purely for logging
    Logging(Logging),

    /// Pre test for the 128 cipher limit
    ///
    /// Some servers have either a ClientHello total size limit or a 128 cipher limit (e.g. old ASAs)
    PreTest128,

    /// `SSLv...` | `TLS...`: Offered protocols
    Protocol(Protocol),

    /// `NPN`: Is NPN offered?
    Npn,

    /// `ALPN`: Is Alpn offered?
    Alpn,

    /// `ALPN_HTTP2"`: Is Alpn offered?
    AlpnHttp2,

    /// `cipherlist_...`: Offered cipher categories
    CipherList(CipherList),

    /// Tests Perfect Forward Secrecy
    Pfs(Option<Pfs>),

    /// `protocol_negotiated`: Default protocol used by server
    DefaultProtocol,

    /// `cipher_negotiated`: Default cipher used by server
    DefaultCipher,

    /// `TLS_session_ticket`: Check the session ticket's lifetime
    TlsSessionTicket,

    /// `cipher_order` | `cipher_order_...` | `cipherorder_...`: Ordered list of ciphers use by each protocol
    CipherOrder(Option<Protocol>),

    /// Information about the used certificate(s)
    ///
    /// They will be numbered if the server uses more than one certificate.
    ///
    /// The number is reported by [`Logging::CertCount`].
    Certificate(Cert, Option<NonZeroUsize>),

    /// `insecure_redirect`: Redirect to an http url
    InsecureRedirect,

    /// `ipv4_in_header`: An ipv4 address has been found in the http header
    Ipv4InHeader,

    /// Security related http header
    ///
    /// The `bool` flag indicates whether the header is set more than once.
    ///
    /// [`HttpHeader::StrictTransportSecurity`] will only be reported with `true`
    /// as it is checked in depth in [`FindingId::Hsts`]
    HttpHeader(HttpHeader, bool),

    /// `security_headers`: Absence of security related http header
    SecurityHeaders,

    /// HTTP Strict Transport Security parameters
    Hsts(Option<Hsts>),

    ///HTTP Public Key Pinning
    Hpkp(Option<Hpkp>),

    /// `cookie_...`: Http cookies
    Cookie(Cookie),

    /// Test of known vulnerabilities
    Vulnerabilities(Vulnerabilities),

    /// Testing individual ciphers identified by hexcode
    ///
    /// Might be tested per (and then grouped by) [`Protocol`]
    Cipher(Option<Protocol>, String),

    /// Tls clients simulated by testssl
    ///
    /// May contain a `short` name of the simulated client.
    /// The list of clients can be found in `$TESTSSL_INSTALL_DIR/etc/client-simulation.txt`.
    ClientSimulation(Option<String>),

    /// An id which has not been categorized yet
    Unknown(String),
}

/// Perfect Forward Secrecy
pub enum Pfs {
    /// `PFS_ciphers`: Offered ciphers
    Ciphers,

    /// `PFS_ECDHE_curves`: Offered elliptic curves
    Curves,

    /// `DH_groups`: Offered Diffie Hellman groups
    Groups,
}

/// Checks about http cookies
pub enum Cookie {
    /// `cookie_bigip_f5`: Cookie set by [BIG-IP](https://www.f5.com/products/big-ip-services)
    BigIp,
    /// `cookie_count`: Number of cookies
    Count,
    /// `cookie_secure`: Ratio of secure cookies
    Secure,
    /// `cookie_httponly`: Ratio of http only cookies
    HttpOnly,
}

/// Ids which are purely for logging
pub enum Logging {
    /// `optimal_proto`: Determines the optimal protocol to connect to the server with,
    ///
    /// before running service detection and any other checks
    OptimalProtocol,

    /// `service`: The service detection's result
    Service,

    /// `host_certificate_Problem`: Could not retrieve host cert
    CertProblem,

    /// `TLS_timestamp`: The tls server's reported time
    TlsClockSkew,

    /// `TLS_extensions`: Tls extensions supported by server
    TlsExtensions,

    /// `SSL_sessionID_support`: Is ssl's session ID supported?
    SslSessionId,

    /// `sessionresumption_ticket`
    SessionResumptionTicket,

    /// `sessionresumption_ID`
    SessionResumptionId,

    /// `cert_numbers`: Number of certificates found
    CertCount,

    /// `http_get`: A utility function used to query revoked certificates
    HttpGet,

    /// `HTTP_status_code`: Queries the http headers
    HttpHeader,

    /// `HTTP_clock_skew`: The http server's reported time
    HttpClockSkew,

    /// `HTTP_headerAge`: The `Age` header indicating caching
    HttpAge,

    /// `banner_server`: Retrieves the server's banner
    ServerBanner,

    /// `banner_application`: Retrieves the application's banner
    ApplicationBanner,

    /// `banner_reverseproxy`: Retrieves the reverse proxy's banner
    ProxyBanner,
}

/// Http Header retrieved by testssl
pub enum HttpHeader {
    /// `X-Frame-Options`
    XFrameOptions,
    /// `X-Content-Type-Options`
    XContentTypeOptions,
    /// `Content-Security-Policy`
    ContentSecurityPolicy,
    /// `X-Content-Security-Policy`
    XContentSecurityPolicy,
    /// `X-WebKit-CSP`
    XWebKitCSP,
    /// `Content-Security-Policy-Report-Only`
    ContentSecurityPolicyReportOnly,
    /// `Expect-CT`
    ExpectCT,
    /// `Access-Control-Allow-Origin`
    AccessControlAllowOrigin,
    /// `Upgrade`
    Upgrade,
    /// `X-Served-By`
    XServedBy,
    /// `Referrer-Policy`
    ReferrerPolicy,
    /// `X-UA-Compatible`
    XUACompatible,
    /// `Cache-Control`
    CacheControl,
    /// `Pragma`
    Pragma,
    /// `X-XSS-Protection`
    XXSSProtection,
    /// `Strict-Transport-Security`
    StrictTransportSecurity,
}

/// Cipher categories
pub enum CipherList {
    /// `NULL`
    Null,
    /// `aNULL`
    ANull,
    /// `EXPORT`
    Export,
    /// `LOW`
    Low,
    /// `3DES_IDEA`
    TripleDes,
    /// `AVERAGE`
    Average,
    /// `STRONG`
    Strong,
}

/// Protocol used to secure a connection
pub enum Protocol {
    /// `SSLv2`
    SSL2,
    /// `SSLv3`
    SSL3,
    /// `TLS1`
    TLS1,
    /// `TLS1_1`
    TLS11,
    /// `TLS1_2`
    TLS12,
    /// `TLS1_3`
    TLS13,
}

/// Known vulnerabilities which are tested
pub enum Vulnerabilities {
    /// `heartbleed`: Buffer overflow which allows reading of RAM
    Heartbleed,

    /// `CCS`: Bad "ChangeCipherSpec" implementation allows mitm decoding entire session if hijacked at initiation.
    CCS,

    /// `ticketbleed`: Read 31 uninitialized bytes from a F5 BIG-IP application
    Ticketbleed,

    /// `ROBOT`: Allows breaking RSA on a set of recorded messages with affordable resources
    Robot,

    /// `secure_renego`: Secure Renegotiation (RFC 5746) usually consumes more resources on the server than on the client
    SecRen,

    /// `secure_client_renego`: Denial of service by triggering a resource intensive secure renegotiation from the client
    SecCliRen,

    /// `CRIME_TLS`: Info leak through observable changes in compression
    CRIME,

    /// `BREACH`: Specialized version of [`Vulnerabilities::CRIME`]
    BREACH,

    /// `POODLE_SSL`
    PoodleSsl,
    /// `POODLE_TLS`
    PoodleTls,
    /// `fallback_SCSV`
    FallbackSCSV,
    /// `SWEET32`
    Sweet32,
    /// `FREAK`
    Freak,
    /// `DROWN`
    Drown,
    /// `DROWN_hint`
    DrownHint,
    /// `LOGJAM`
    LogJam,
    /// `LOGJAM-common_primes`
    LogJamCommonPrimes,
    /// `BEAST`
    Beast,
    /// `BEAST_CBC_SSL3`
    BeastSsl3,
    /// `BEAST_CBC_TLS1`
    BeastTls1,
    /// `LUCKY13`
    Lucky13,
    /// `RC4`
    Rc4,
    /// `GREASE`
    Grease,
}

/// HTTP Strict Transport Security's parameters
pub enum Hsts {
    /// `HSTS_time`: HTTP Strict Transport Security's `max-age` parameter
    MaxAge,
    /// `HSTS_subdomains`: HTTP Strict Transport Security's `includeSubDomains` parameter
    Subdomains,
    /// `HSTS_preload`: HTTP Strict Transport Security's `preload` parameter
    Preload,
}

/// HTTP Public Key Pinning's parameters
pub enum Hpkp {
    /// `HPKP_error`: Multiple `Public-Key-Pins` in http header
    Multiple,
    /// `HPKP_notice`: Multiple `Public-Key-Pins-Report-Only` in header
    MultipleReportOnly,
    /// `HPKP_age`: HTTP Public Key Pinning's `max-age` parameter
    MaxAge,
    /// `HPKP_subdomains`: HTTP Public Key Pinning's `includeSubDomains` parameter
    Subdomains,
    /// `HPKP_preload`: HTTP Public Key Pinning's `preload` parameter
    Preload,
    /// `HPKP_SPKIs`: Number of spkis
    Spkis,
    /// `HPKP_...`: Information about a specific spki
    Spki(String),
    /// `HPKP_SPKImatch`: Does an spki match the host's certificate?
    SpkiMatch,
    /// `HPKP_backup`: Missing backup keys
    Backup,
}

/// Information about the used certificate(s)
pub enum Cert {
    /// `cert`
    Cert,
    /// `cert_mustStapleExtension`
    MustStapleExtension,
    /// `cert_signatureAlgorithm`
    SignatureAlgorithm,
    /// `cert_keySize`
    KeySize,
    /// `cert_keyUsage`
    KeyUsage,
    /// `cert_extKeyUsage`
    ExtKeyUsage,
    /// `cert_serialNumber`
    SerialNumber,
    /// `cert_serialNumberLen`
    SerialNumberLen,
    /// `cert_fingerprintSHA1`
    FingerprintSHA1,
    /// `cert_fingerprintSHA256`
    FingerprintSHA256,
    /// `cert_commonName`
    CommonName,
    /// `cert_commonName_wo_SNI`
    CommonNameWithoutSNI,
    /// `cert_subjectAltName`
    SubjectAltName,
    /// `cert_caIssuers`
    CAIssuers,
    /// `cert_trust`
    Trust,
    /// `cert_chain_of_trust`
    ChainOfTrust,
    /// `cert_certificatePolicies_EV`
    CertificatePoliciesEV,
    /// `cert_eTLS`
    ETS,
    /// `cert_expirationStatus`
    ExpirationStatus,
    /// `cert_notBefore`
    NotBefore,
    /// `cert_notAfter`
    NotAfter,
    /// `cert_validityPeriod`
    ValidityPeriod,
    /// `certs_countServer`
    CertsCountServer,
    /// `certs_list_ordering_problem`
    CertsListOrderingProblem,
    /// `pwnedkeys`
    PwnedKeys,
    /// `cert_crlDistributionPoints`
    CRLDistributionPoints,
    /// `cert_ocspURL`
    OCSPUrl,
    /// `cert_revocation`
    Revocation,
    /// `OCSP_stapling`
    OCSPStapling,
    /// `DNS_CAArecord`
    CAARecord,
    /// `cert_crlRevoked`
    CRLRevoked,
    /// `cert_ocspRevoked`
    OCSPRevoked,
}

/// Issues with your testssl installation
pub enum ScanIssues {
    /// `scanProblem`: an error occurred during the scan
    ScanProblem,

    /// `old_fart` | `too_old_openssl`: your openssl version is way too old
    OldOpenssl,

    /// `engine_problem`: No engine or GOST support via engine with your openssl
    EngineProblem,
}

impl From<&str> for FindingId {
    fn from(s: &str) -> Self {
        use FindingId::{
            Certificate as crt, CipherList as cl, CipherOrder as co, Cookie as c, Hpkp as kp,
            Hsts as ts, HttpHeader as hh, Logging as l, Pfs as pfs, Protocol as p,
            ScanIssues as si, Vulnerabilities as v,
        };
        const T: bool = true;
        const F: bool = false;

        match s {
            "http_get" => l(Logging::HttpGet),
            "service" => l(Logging::Service),
            "HTTP_status_code" => l(Logging::HttpHeader),
            "insecure_redirect" => Self::InsecureRedirect,
            "ipv4_in_header" => Self::Ipv4InHeader,
            "HTTP_clock_skew" => l(Logging::HttpClockSkew),
            "HTTP_headerAge" => l(Logging::HttpAge),
            "HSTS_time" => ts(Some(Hsts::MaxAge)),
            "HSTS_subdomains" => ts(Some(Hsts::Subdomains)),
            "HSTS_preload" => ts(Some(Hsts::Preload)),
            "HSTS" => ts(None),
            "HPKP_error" => kp(Some(Hpkp::Multiple)),
            "HPKP_notice" => kp(Some(Hpkp::MultipleReportOnly)),
            "HPKP_SPKIs" => kp(Some(Hpkp::Spkis)),
            "HPKP_age" => kp(Some(Hpkp::MaxAge)),
            "HPKP_subdomains" => kp(Some(Hpkp::Subdomains)),
            "HPKP_preload" => kp(Some(Hpkp::Preload)),
            "HPKP_SPKImatch" => kp(Some(Hpkp::SpkiMatch)),
            "HPKP_backup" => kp(Some(Hpkp::Backup)),
            "HPKP" => kp(None),
            "banner_server" => l(Logging::ServerBanner),
            "banner_application" => l(Logging::ApplicationBanner),
            "banner_reverseproxy" => l(Logging::ProxyBanner),
            "cookie_bigip_f5" => c(Cookie::BigIp),
            "cookie_count" => c(Cookie::Count),
            "cookie_secure" => c(Cookie::Secure),
            "cookie_httponly" => c(Cookie::HttpOnly),
            "X-Frame-Options" => hh(HttpHeader::XFrameOptions, F),
            "X-Content-Type-Options" => hh(HttpHeader::XContentTypeOptions, F),
            "Content-Security-Policy" => hh(HttpHeader::ContentSecurityPolicy, F),
            "X-Content-Security-Policy" => hh(HttpHeader::XContentSecurityPolicy, F),
            "X-WebKit-CSP" => hh(HttpHeader::XWebKitCSP, F),
            "Content-Security-Policy-Report-Only" => {
                hh(HttpHeader::ContentSecurityPolicyReportOnly, F)
            }
            "Expect-CT" => hh(HttpHeader::ExpectCT, F),
            "Access-Control-Allow-Origin" => hh(HttpHeader::AccessControlAllowOrigin, F),
            "Upgrade" => hh(HttpHeader::Upgrade, F),
            "X-Served-By" => hh(HttpHeader::XServedBy, F),
            "Referrer-Policy" => hh(HttpHeader::ReferrerPolicy, F),
            "X-UA-Compatible" => hh(HttpHeader::XUACompatible, F),
            "Cache-Control" => hh(HttpHeader::CacheControl, F),
            "Pragma" => hh(HttpHeader::Pragma, F),
            "X-XSS-Protection" => hh(HttpHeader::XXSSProtection, F),
            "X-Frame-Options_multiple" => hh(HttpHeader::XFrameOptions, T),
            "X-Content-Type-Options_multiple" => hh(HttpHeader::XContentTypeOptions, T),
            "Content-Security-Policy_multiple" => hh(HttpHeader::ContentSecurityPolicy, T),
            "X-Content-Security-Policy_multiple" => hh(HttpHeader::XContentSecurityPolicy, T),
            "X-WebKit-CSP_multiple" => hh(HttpHeader::XWebKitCSP, T),
            "Content-Security-Policy-Report-Only_multiple" => {
                hh(HttpHeader::ContentSecurityPolicyReportOnly, T)
            }
            "Expect-CT_multiple" => hh(HttpHeader::ExpectCT, T),
            "Access-Control-Allow-Origin_multiple" => hh(HttpHeader::AccessControlAllowOrigin, T),
            "Upgrade_multiple" => hh(HttpHeader::Upgrade, T),
            "X-Served-By_multiple" => hh(HttpHeader::XServedBy, T),
            "Referrer-Policy_multiple" => hh(HttpHeader::ReferrerPolicy, T),
            "X-UA-Compatible_multiple" => hh(HttpHeader::XUACompatible, T),
            "Cache-Control_multiple" => hh(HttpHeader::CacheControl, T),
            "Pragma_multiple" => hh(HttpHeader::Pragma, T),
            "X-XSS-Protection_multiple" => hh(HttpHeader::XXSSProtection, T),
            "Strict-Transport-Security_multiple" => hh(HttpHeader::StrictTransportSecurity, T),
            "security_headers" => Self::SecurityHeaders,
            "SSLv2" => p(Protocol::SSL2),
            "SSLv3" => p(Protocol::SSL3),
            "TLS1" => p(Protocol::TLS1),
            "TLS1_1" => p(Protocol::TLS11),
            "TLS1_2" => p(Protocol::TLS12),
            "TLS1_3" => p(Protocol::TLS13),
            "cipherlist_NULL" => cl(CipherList::Null),
            "cipherlist_aNULL" => cl(CipherList::ANull),
            "cipherlist_EXPORT" => cl(CipherList::Export),
            "cipherlist_LOW" => cl(CipherList::Low),
            "cipherlist_3DES_IDEA" => cl(CipherList::TripleDes),
            "cipherlist_AVERAGE" => cl(CipherList::Average),
            "cipherlist_STRONG" => cl(CipherList::Strong),
            "cipher_order" => co(None),
            "cipher_order_SSLV2" => co(Some(Protocol::SSL2)),
            "cipher_order_SSLV3" => co(Some(Protocol::SSL3)),
            "cipher_order_TLSv1" => co(Some(Protocol::TLS1)),
            "cipher_order_TLSv1.1" => co(Some(Protocol::TLS11)),
            "cipher_order_TLSv1.2" => co(Some(Protocol::TLS12)),
            "cipher_order_TLSv1.3" => co(Some(Protocol::TLS13)),
            "cipherorder_SSLV2" => co(Some(Protocol::SSL2)),
            "cipherorder_SSLV3" => co(Some(Protocol::SSL3)),
            "cipherorder_TLSv1" => co(Some(Protocol::TLS1)),
            "cipherorder_TLSv1_1" => co(Some(Protocol::TLS11)),
            "cipherorder_TLSv1_2" => co(Some(Protocol::TLS12)),
            "cipherorder_TLSv1_3" => co(Some(Protocol::TLS13)),
            "protocol_negotiated" => Self::DefaultProtocol,
            "cipher_negotiated" => Self::DefaultCipher,
            "host_certificate_Problem" => l(Logging::CertProblem),
            "TLS_timestamp" => l(Logging::TlsClockSkew),
            "TLS_extensions" => l(Logging::TlsExtensions),
            "TLS_session_ticket" => Self::TlsSessionTicket,
            "SSL_sessionID_support" => l(Logging::SslSessionId),
            "sessionresumption_ticket" => l(Logging::SessionResumptionTicket),
            "sessionresumption_ID" => l(Logging::SessionResumptionId),
            "PFS" => pfs(None),
            "PFS_ciphers" => pfs(Some(Pfs::Ciphers)),
            "PFS_ECDHE_curves" => pfs(Some(Pfs::Curves)),
            "DH_groups" => pfs(Some(Pfs::Groups)),
            "NPN" => Self::Npn,
            "ALPN" => Self::Alpn,
            "ALPN_HTTP2" => Self::AlpnHttp2,
            "heartbleed" => v(Vulnerabilities::Heartbleed),
            "CCS" => v(Vulnerabilities::CCS),
            "ticketbleed" => v(Vulnerabilities::Ticketbleed),
            "secure_renego" => v(Vulnerabilities::SecRen),
            "secure_client_renego" => v(Vulnerabilities::SecCliRen),
            "CRIME_TLS" => v(Vulnerabilities::CRIME),
            "BREACH" => v(Vulnerabilities::BREACH),
            "SWEET32" => v(Vulnerabilities::Sweet32),
            "POODLE_SSL" => v(Vulnerabilities::PoodleSsl),
            "POODLE_TLS" => v(Vulnerabilities::PoodleTls),
            "fallback_SCSV" => v(Vulnerabilities::FallbackSCSV),
            "FREAK" => v(Vulnerabilities::Freak),
            "LOGJAM" => v(Vulnerabilities::LogJam),
            "LOGJAM-common_primes" => v(Vulnerabilities::LogJamCommonPrimes),
            "DROWN" => v(Vulnerabilities::Drown),
            "DROWN_hint" => v(Vulnerabilities::DrownHint),
            "BEAST" => v(Vulnerabilities::Beast),
            "BEAST_CBC_SSL3" => v(Vulnerabilities::BeastSsl3),
            "BEAST_CBC_TLS1" => v(Vulnerabilities::BeastTls1),
            "LUCKY13" => v(Vulnerabilities::Lucky13),
            "RC4" => v(Vulnerabilities::Rc4),
            "GREASE" => v(Vulnerabilities::Grease),
            "ROBOT" => v(Vulnerabilities::Robot),
            "scanProblem" => si(ScanIssues::ScanProblem),
            "optimal_proto" => l(Logging::OptimalProtocol),
            "pre_128cipher" => Self::PreTest128,
            "old_fart" => si(ScanIssues::OldOpenssl),
            "too_old_openssl" => si(ScanIssues::OldOpenssl),
            "engine_problem" => si(ScanIssues::EngineProblem),
            "cert_numbers" => l(Logging::CertCount),
            "cert" => crt(Cert::Cert, None),
            "cert_mustStapleExtension" => crt(Cert::MustStapleExtension, None),
            "cert_signatureAlgorithm" => crt(Cert::SignatureAlgorithm, None),
            "cert_keySize" => crt(Cert::KeySize, None),
            "cert_keyUsage" => crt(Cert::KeyUsage, None),
            "cert_extKeyUsage" => crt(Cert::ExtKeyUsage, None),
            "cert_serialNumber" => crt(Cert::SerialNumber, None),
            "cert_serialNumberLen" => crt(Cert::SerialNumberLen, None),
            "cert_fingerprintSHA1" => crt(Cert::FingerprintSHA1, None),
            "cert_fingerprintSHA256" => crt(Cert::FingerprintSHA256, None),
            "cert_commonName" => crt(Cert::CommonName, None),
            "cert_commonName_wo_SNI" => crt(Cert::CommonNameWithoutSNI, None),
            "cert_subjectAltName" => crt(Cert::SubjectAltName, None),
            "cert_caIssuers" => crt(Cert::CAIssuers, None),
            "cert_trust" => crt(Cert::Trust, None),
            "cert_chain_of_trust" => crt(Cert::ChainOfTrust, None),
            "cert_certificatePolicies_EV" => crt(Cert::CertificatePoliciesEV, None),
            "cert_eTLS" => crt(Cert::ETS, None),
            "cert_expirationStatus" => crt(Cert::ExpirationStatus, None),
            "cert_notBefore" => crt(Cert::NotBefore, None),
            "cert_notAfter" => crt(Cert::NotAfter, None),
            "cert_validityPeriod" => crt(Cert::ValidityPeriod, None),
            "certs_countServer" => crt(Cert::CertsCountServer, None),
            "certs_list_ordering_problem" => crt(Cert::CertsListOrderingProblem, None),
            "pwnedkeys" => crt(Cert::PwnedKeys, None),
            "cert_crlDistributionPoints" => crt(Cert::CRLDistributionPoints, None),
            "cert_ocspURL" => crt(Cert::OCSPUrl, None),
            "cert_revocation" => crt(Cert::Revocation, None),
            "OCSP_stapling" => crt(Cert::OCSPStapling, None),
            "DNS_CAArecord" => crt(Cert::CAARecord, None),
            "cert_crlRevoked" => crt(Cert::CRLRevoked, None),
            "cert_ocspRevoked" => crt(Cert::OCSPRevoked, None),
            "clientsimulation" => Self::ClientSimulation(None),
            _ => {
                if let Some((test, number)) = s.split_once(" <cert#") {
                    let cert = match test {
                        "cert" => Some(Cert::Cert),
                        "cert_mustStapleExtension" => Some(Cert::MustStapleExtension),
                        "cert_signatureAlgorithm" => Some(Cert::SignatureAlgorithm),
                        "cert_keySize" => Some(Cert::KeySize),
                        "cert_keyUsage" => Some(Cert::KeyUsage),
                        "cert_extKeyUsage" => Some(Cert::ExtKeyUsage),
                        "cert_serialNumber" => Some(Cert::SerialNumber),
                        "cert_serialNumberLen" => Some(Cert::SerialNumberLen),
                        "cert_fingerprintSHA1" => Some(Cert::FingerprintSHA1),
                        "cert_fingerprintSHA256" => Some(Cert::FingerprintSHA256),
                        "cert_commonName" => Some(Cert::CommonName),
                        "cert_commonName_wo_SNI" => Some(Cert::CommonNameWithoutSNI),
                        "cert_subjectAltName" => Some(Cert::SubjectAltName),
                        "cert_caIssuers" => Some(Cert::CAIssuers),
                        "cert_trust" => Some(Cert::Trust),
                        "cert_chain_of_trust" => Some(Cert::ChainOfTrust),
                        "cert_certificatePolicies_EV" => Some(Cert::CertificatePoliciesEV),
                        "cert_eTLS" => Some(Cert::ETS),
                        "cert_expirationStatus" => Some(Cert::ExpirationStatus),
                        "cert_notBefore" => Some(Cert::NotBefore),
                        "cert_notAfter" => Some(Cert::NotAfter),
                        "cert_validityPeriod" => Some(Cert::ValidityPeriod),
                        "certs_countServer" => Some(Cert::CertsCountServer),
                        "certs_list_ordering_problem" => Some(Cert::CertsListOrderingProblem),
                        "pwnedkeys" => Some(Cert::PwnedKeys),
                        "cert_crlDistributionPoints" => Some(Cert::CRLDistributionPoints),
                        "cert_ocspURL" => Some(Cert::OCSPUrl),
                        "cert_revocation" => Some(Cert::Revocation),
                        "OCSP_stapling" => Some(Cert::OCSPStapling),
                        "DNS_CAArecord" => Some(Cert::CAARecord),
                        "cert_crlRevoked" => Some(Cert::CRLRevoked),
                        "cert_ocspRevoked" => Some(Cert::OCSPRevoked),
                        _ => None,
                    };
                    let number = number
                        .strip_suffix('>')
                        .map(NonZeroUsize::from_str)
                        .and_then(Result::ok);
                    if let Some((cert, number)) = cert.zip(number) {
                        return crt(cert, Some(number));
                    }
                }
                if let Some(client) = s.strip_prefix("clientsimulation-") {
                    return Self::ClientSimulation(Some(client.to_string()));
                }
                if let Some(spki) = s.strip_prefix("HPKP_") {
                    return Self::Hpkp(Some(Hpkp::Spki(spki.to_string())));
                }
                if let Some(hexcode) = s.strip_prefix("cipher_x") {
                    return Self::Cipher(None, hexcode.to_string());
                }
                if let Some(string) = s.strip_prefix("cipher-") {
                    if let Some((proto, hexcode)) = string.split_once("_x") {
                        let proto = match proto {
                            "ssl2" => Some(Protocol::SSL2),
                            "ssl3" => Some(Protocol::SSL3),
                            "tls1" => Some(Protocol::TLS1),
                            "tls1_1" => Some(Protocol::TLS11),
                            "tls1_2" => Some(Protocol::TLS12),
                            "tls1_3" => Some(Protocol::TLS13),
                            _ => None,
                        };
                        if proto.is_some() {
                            return Self::Cipher(proto, hexcode.to_string());
                        }
                    }
                }
                Self::Unknown(s.to_string())
            }
        }
    }
}
