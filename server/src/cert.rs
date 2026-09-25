use anyhow::{Result, anyhow};
use rcgen::{CertificateParams, DistinguishedName, DnType};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::time::{Duration, SystemTime};

pub fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let mut params = CertificateParams::new(vec!["localhost".to_string()])?;
    
    // Set certificate parameters
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CommonName, "WebTransport Server");
    params.distinguished_name.push(DnType::OrganizationName, "WebTransport Test");
    
    // Set validity period
    params.not_before = SystemTime::now().into();
    params.not_after = (SystemTime::now() + Duration::from_secs(365 * 24 * 3600)).into(); // 1 year
    
    // Add subject alternative names
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName("localhost".try_into()?),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        rcgen::SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
    ];
    
    // Generate key pair and certificate
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    
    // Convert to DER format
    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der = PrivateKeyDer::from(PrivatePkcs8KeyDer::from(key_pair.serialize_der()));
    
    Ok((cert_der, key_der))
}
