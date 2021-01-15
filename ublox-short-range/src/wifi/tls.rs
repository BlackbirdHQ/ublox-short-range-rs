use atat::AtatClient;
use heapless::{consts, ArrayLength, String};
use crate::{
    error::Error, 
    UbloxClient,
    command::security::{
        *,
        types::*,
    },
    socket::{SocketHandle, SocketType, TcpSocket},
};

pub trait TLS {
    fn import_certificate(
        &self,
        name: &str,
        certificate: &[u8],
    ) -> Result<(), Error>;
    fn import_root_ca(&self,
        name: &str,
        root_ca: &[u8]
    ) -> Result<(), Error>;
    fn import_private_key(
        &self,
        name: &str,
        private_key: &[u8],
        password: Option<&str>,
    ) -> Result<(), Error>;
    fn enable_tls(&self, 
        socket: SocketHandle, 
        ca_cert_name: Option<&str>, 
        client_cert_name: Option<&str>, 
        priv_key_name: Option<&str>,
    ) -> Result<(), Error>;
}

impl<C, N, L> TLS for UbloxClient<C, N, L>
where
    C: atat::AtatClient,
    N: ArrayLength<Option<crate::sockets::SocketSetItem<L>>>,
    L: ArrayLength<u8>,
{
    fn import_certificate(
        &self,
        name: &str,
        certificate: &[u8],
    ) -> Result<(), Error> {
        assert!(name.len() < 200);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::ClientCertificate,
            data_size: certificate.len(),
            internal_name: name,
            password: None,
        })?;

        self.send_at(SendSecurityDataImport {
            data: serde_at::ser::Bytes(certificate),
        })?;

        //Check MD5?

        Ok(())
    }

    fn import_root_ca(&self, name: &str, root_ca: &[u8]) -> Result<(), Error> {
        assert!(name.len() < 200);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::TrustedRootCA,
            data_size: root_ca.len(),
            internal_name: name,
            password: None,
        })?;

        self.send_at(SendSecurityDataImport {
            data: serde_at::ser::Bytes(root_ca),
        })?;

        //Check MD5?

        Ok(())
    }

    fn import_private_key(
        &self,
        name: &str,
        private_key: &[u8],
        password: Option<&str>,
    ) -> Result<(), Error> {
        assert!(name.len() < 200);

        self.send_at(PrepareSecurityDataImport {
            data_type: SecurityDataType::ClientPrivateKey,
            data_size: private_key.len(),
            internal_name: name,
            password,
        })?;

        self.send_at(SendSecurityDataImport {
            data: serde_at::ser::Bytes(private_key),
        })?;

        //Check MD5?

        Ok(())
    }

    fn enable_tls(
        &self, 
        socket: SocketHandle,
        ca_cert_name: Option<&str>, 
        client_cert_name: Option<&str>, 
        priv_key_name: Option<&str>
    ) -> Result<(), Error> {
        //Change socket handle to do TLS now, 
        //Needs name of Certificates.
        let mut sockets = self.sockets.try_borrow_mut()?;
        match sockets.socket_type(socket) {
            Some(SocketType::Tcp) => {
                let mut tcp = sockets.get::<TcpSocket<_>>(socket)?;
                if let Some(ca) = ca_cert_name{
                    tcp.ca_cert_name =  Some(String::from(ca));
                }
                if let Some(cert) = client_cert_name{
                    tcp.c_cert_name =  Some(String::from(cert));
                }
                if let Some(key) = priv_key_name{
                    tcp.c_key_name =  Some(String::from(key));
                }
            }
            _ => return Err(Error::SocketNotFound),
        }
        Ok(())
    }
}