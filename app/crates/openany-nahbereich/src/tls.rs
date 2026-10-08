//! TLS zwischen zwei Geraeten -- beide mit selbst erzeugtem Zertifikat.
//!
//! Es gibt keine Stelle, die Zertifikate ausstellt. Die Identitaet eines
//! Geraets ist der SHA-256-Fingerabdruck seines Zertifikats (wie bei
//! LocalSend). Geprueft wird deshalb zweierlei: dass das Zertifikat in sich
//! gueltig ist, und -- wo bekannt -- dass sein Fingerabdruck der erwartete ist.
//!
//! Nach dem Vorbild von LocalSends `PinnedServerCertVerifier` und
//! `CustomClientCertVerifier` (Apache-2.0), die dort nicht oeffentlich sind.

use localsend::crypto::cert::{fingerprint_from_cert_der, verify_cert_from_der};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{
    ClientConfig, DigitallySignedStruct, DistinguishedName, Error, ServerConfig, SignatureScheme,
};
use std::sync::Arc;

fn anbieter() -> Arc<CryptoProvider> {
    Arc::new(ring::default_provider())
}

fn unpassend(grund: &str) -> Error {
    Error::General(grund.to_string())
}

/// Prueft ein Zertifikat in sich und optional gegen einen erwarteten
/// Fingerabdruck.
#[derive(Debug)]
struct Pruefer {
    anbieter: Arc<CryptoProvider>,
    erwartet: Option<String>,
}

impl Pruefer {
    fn pruefen(&self, zertifikat: &CertificateDer<'_>) -> Result<(), Error> {
        verify_cert_from_der(zertifikat.as_ref(), None).map_err(|e| unpassend(&e.to_string()))?;
        if let Some(fp) = &self.erwartet {
            if !fingerprint_from_cert_der(zertifikat.as_ref()).eq_ignore_ascii_case(fp) {
                return Err(unpassend(
                    "The other device does not have the expected fingerprint.",
                ));
            }
        }
        Ok(())
    }

    fn tls12(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls12_signature(m, c, d, &self.anbieter.signature_verification_algorithms)
    }

    fn tls13(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls13_signature(m, c, d, &self.anbieter.signature_verification_algorithms)
    }

    fn schemata(&self) -> Vec<SignatureScheme> {
        self.anbieter
            .signature_verification_algorithms
            .supported_schemes()
    }
}

impl ServerCertVerifier for Pruefer {
    fn verify_server_cert(
        &self,
        e: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.pruefen(e)?;
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls12(m, c, d)
    }
    fn verify_tls13_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls13(m, c, d)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemata()
    }
}

impl ClientCertVerifier for Pruefer {
    fn offer_client_auth(&self) -> bool {
        true
    }
    fn client_auth_mandatory(&self) -> bool {
        true
    }
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
    fn verify_client_cert(
        &self,
        e: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        self.pruefen(e)?;
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls12(m, c, d)
    }
    fn verify_tls13_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.tls13(m, c, d)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemata()
    }
}

fn zertifikat_und_schluessel(
    zert: &str,
    schluessel: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Error> {
    let kette = CertificateDer::pem_slice_iter(zert.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| unpassend(&e.to_string()))?;
    let key = PrivateKeyDer::from_pem_slice(schluessel.as_bytes())
        .map_err(|e| unpassend(&e.to_string()))?;
    Ok((kette, key))
}

/// Der Dienst: verlangt von jedem Anrufer ein gueltiges Zertifikat. WER
/// anruft (gepaart oder nicht), entscheidet der Dienst danach anhand des
/// Fingerabdrucks.
pub fn dienst_konfiguration(zert: &str, schluessel: &str) -> Result<ServerConfig, Error> {
    let (kette, key) = zertifikat_und_schluessel(zert, schluessel)?;
    let a = anbieter();
    ServerConfig::builder_with_provider(a.clone())
        .with_safe_default_protocol_versions()?
        .with_client_cert_verifier(Arc::new(Pruefer {
            anbieter: a,
            erwartet: None,
        }))
        .with_single_cert(kette, key)
}

/// Der Anrufer: zeigt sein Zertifikat und besteht auf dem erwarteten
/// Fingerabdruck am anderen Ende.
pub fn anruf_konfiguration(
    zert: &str,
    schluessel: &str,
    erwartet: &str,
) -> Result<ClientConfig, Error> {
    let (kette, key) = zertifikat_und_schluessel(zert, schluessel)?;
    let a = anbieter();
    ClientConfig::builder_with_provider(a.clone())
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Pruefer {
            anbieter: a,
            erwartet: Some(erwartet.to_string()),
        }))
        .with_client_auth_cert(kette, key)
}

pub fn fingerabdruck(zertifikat: &CertificateDer<'_>) -> String {
    fingerprint_from_cert_der(zertifikat.as_ref())
}
