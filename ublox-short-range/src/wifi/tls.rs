use crate::{
    command::edm::BigEdmAtCmdWrapper,
    command::security::{types::*, *},
    error::Error,
    UbloxClient,
};
use core::convert::TryInto;
use embedded_hal::digital::OutputPin;
use embedded_time::duration::{Generic, Milliseconds};
use embedded_time::Clock;
use heapless::String;

pub trait TLS {
    fn import_certificate(&mut self, name: &str, certificate: &[u8]) -> Result<(), Error>;
    fn import_root_ca(&mut self, name: &str, root_ca: &[u8]) -> Result<(), Error>;
    fn import_private_key(
        &mut self,
        name: &str,
        private_key: &[u8],
        password: Option<&str>,
    ) -> Result<(), Error>;
}

impl<C, CLK, RST, const N: usize, const L: usize> TLS for UbloxClient<C, CLK, RST, N, L>
where
    C: atat::AtatClient,
    CLK: Clock,
    RST: OutputPin,
    Generic<CLK::T>: TryInto<Milliseconds>,
{
    /// Importing credentials enabeles their use for all further TCP connections
    fn import_certificate(&mut self, name: &str, certificate: &[u8]) -> Result<(), Error> {
        assert!(name.len() < 16);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::ClientCertificate,
            data_size: certificate.len(),
            internal_name: name,
            password: None,
        })?;

        self.send_internal(
            &BigEdmAtCmdWrapper(SendSecurityDataImport {
                data: atat::serde_at::ser::Bytes(certificate),
            }),
            false,
        )?;

        self.security_credentials
            .c_cert_name
            .replace(String::from(name));

        Ok(())
    }

    /// Importing credentials enabeles their use for all further TCP connections
    fn import_root_ca(&mut self, name: &str, root_ca: &[u8]) -> Result<(), Error> {
        assert!(name.len() < 16);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::TrustedRootCA,
            data_size: root_ca.len(),
            internal_name: name,
            password: None,
        })?;

        self.send_internal(
            &BigEdmAtCmdWrapper(SendSecurityDataImport {
                data: atat::serde_at::ser::Bytes(root_ca),
            }),
            false,
        )?;

        self.security_credentials
            .ca_cert_name
            .replace(String::from(name));

        Ok(())
    }

    /// Importing credentials enabeles their use for all further TCP connections
    fn import_private_key(
        &mut self,
        name: &str,
        private_key: &[u8],
        password: Option<&str>,
    ) -> Result<(), Error> {
        assert!(name.len() < 16);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::ClientPrivateKey,
            data_size: private_key.len(),
            internal_name: name,
            password,
        })?;

        self.send_internal(
            &BigEdmAtCmdWrapper(SendSecurityDataImport {
                data: atat::serde_at::ser::Bytes(private_key),
            }),
            false,
        )?;

        self.security_credentials
            .c_key_name
            .replace(String::from(name));

        Ok(())
    }
}
