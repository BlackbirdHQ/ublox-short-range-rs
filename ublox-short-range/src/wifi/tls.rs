use crate::{
    client::SecurityCredentials,
    command::edm::BigEdmAtCmdWrapper,
    command::security::{types::*, *},
    error::Error,
    socket::SocketHandle,
    UbloxClient,
};
use core::convert::TryInto;
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
    fn enable_tls(
        &self,
        socket: SocketHandle,
        ca_cert_name: Option<&str>,
        client_cert_name: Option<&str>,
        priv_key_name: Option<&str>,
    ) -> Result<(), Error>;
}

impl<C, CLK, const N: usize, const L: usize> TLS for UbloxClient<C, CLK, N, L>
where
    C: atat::AtatClient,
    CLK: Clock,
    Generic<CLK::T>: TryInto<Milliseconds>,
{
    /// Importing credentials enabeles their use for all further TCP connections
    fn import_certificate(&mut self, name: &str, certificate: &[u8]) -> Result<(), Error> {
        assert!(name.len() < 200);

        if let Some(ref sec) = self.security_credentials {
            if let Some(_) = sec.c_cert_name {
                return Err(Error::DublicateCredentials);
            }
        }

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

        match self.security_credentials {
            Some(ref mut creds) => {
                creds.c_cert_name = Some(String::from(name));
            }
            None => {
                self.security_credentials = Some(SecurityCredentials {
                    c_cert_name: Some(String::from(name)),
                    c_key_name: None,
                    ca_cert_name: None,
                })
            }
        }

        Ok(())
    }

    /// Importing credentials enabeles their use for all further TCP connections
    fn import_root_ca(&mut self, name: &str, root_ca: &[u8]) -> Result<(), Error> {
        assert!(name.len() < 200);

        if let Some(ref sec) = self.security_credentials {
            if let Some(_) = sec.ca_cert_name {
                return Err(Error::DublicateCredentials);
            }
        }

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

        match self.security_credentials {
            Some(ref mut creds) => {
                creds.ca_cert_name = Some(String::from(name));
            }
            None => {
                self.security_credentials = Some(SecurityCredentials {
                    ca_cert_name: Some(String::from(name)),
                    c_key_name: None,
                    c_cert_name: None,
                })
            }
        }

        Ok(())
    }

    /// Importing credentials enabeles their use for all further TCP connections
    fn import_private_key(
        &mut self,
        name: &str,
        private_key: &[u8],
        password: Option<&str>,
    ) -> Result<(), Error> {
        assert!(name.len() < 200);

        if let Some(ref sec) = self.security_credentials {
            if let Some(_) = sec.c_key_name {
                return Err(Error::DublicateCredentials);
            }
        }

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

        match self.security_credentials {
            Some(ref mut creds) => {
                creds.c_key_name = Some(String::from(name));
            }
            None => {
                self.security_credentials = Some(SecurityCredentials {
                    c_key_name: Some(String::from(name)),
                    ca_cert_name: None,
                    c_cert_name: None,
                })
            }
        }

        Ok(())
    }

    fn enable_tls(
        &self,
        _socket: SocketHandle,
        _ca_cert_name: Option<&str>,
        _client_cert_name: Option<&str>,
        _priv_key_name: Option<&str>,
    ) -> Result<(), Error> {
        //Change socket handle to do TLS now,
        //Needs name of Certificates.
        // let mut sockets = self.sockets.try_borrow_mut()?;
        // match sockets.socket_type(socket) {
        //     Some(SocketType::Tcp) => {
        //         let mut tcp = sockets.get::<TcpSocket<_>>(socket)?;
        //         if let Some(ca) = ca_cert_name{
        //             tcp.ca_cert_name =  Some(String::from(ca));
        //         }
        //         if let Some(cert) = client_cert_name{
        //             tcp.c_cert_name =  Some(String::from(cert));
        //         }
        //         if let Some(key) = priv_key_name{
        //             tcp.c_key_name =  Some(String::from(key));
        //         }
        //     }
        //     _ => return Err(Error::SocketNotFound),
        // }
        Err(Error::Unimplemented)
    }
}
