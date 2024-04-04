use std::error::Error;
use std::io;

// TODO: more tests
// The current tests only check the versions of your dependencies to be compatible which is required
// for downcasting the `dyn Error`s.
//
// There should exist sophisticated tests starting dummy servers with known setup,
// to see if the errors reliably report the issues.
//
// For example try talking https with a tcp server producing various responses and check for `NativeTlsError::NotSsl`.

/// Test the compatibility of [`reqwest`]'s and [`native_tls`]'s versions
#[test]
fn valid_native_tls_version() {
    assert!(
        reqwest::ClientBuilder::new()
            .use_preconfigured_tls(native_tls::TlsConnector::new().unwrap())
            .build()
            .is_ok(),
        "The versions for reqwest and native-tls have gone out of sync"
    );
}

/// Test the compatibility of [`native_tls`]'s and [`openssl`]'s versions
#[test]
fn valid_openssl_version() {
    // The following is a weird setup to "construct" a
    // `native_tls::Error(native_tls::imp::Error::Ssl(_, _))`
    // whose `Error::source` should be an `openssl::error::ErrorStack`
    struct Dummy(&'static [u8]);
    impl io::Read for Dummy {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            (&mut self.0).read(buf)
        }
    }
    impl io::Write for Dummy {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let Err(native_tls::HandshakeError::Failure(err)) = native_tls::TlsConnector::new()
        .unwrap()
        .connect("dummy", Dummy(b"this is no a valid client hello"))
    else {
        unreachable!()
    };
    let Some(err) = err.source() else {
        panic!("{err:?}");
    };
    assert!(err.is::<openssl::error::ErrorStack>());
}
