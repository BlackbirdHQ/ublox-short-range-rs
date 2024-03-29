use crate::{
    client::new_socket_num,
    command::data_mode::*,
    command::edm::{EdmAtCmdWrapper, EdmDataCommand},
    wifi::peer_builder::PeerUrlBuilder,
    UbloxClient,
};
use atat::blocking::AtatClient;
use embedded_hal::digital::OutputPin;
/// Handles receiving data from sockets
/// implements TCP and UDP for WiFi client
use embedded_nal::{nb, SocketAddr, TcpClientStack};

use ublox_sockets::{Error, SocketHandle, TcpSocket, TcpState};

use super::EGRESS_CHUNK_SIZE;

impl<'buf, 'sub, AtCl, AtUrcCh, RST, const N: usize, const L: usize> TcpClientStack
    for UbloxClient<'buf, 'sub, AtCl, AtUrcCh, RST, N, L>
where
    'buf: 'sub,
    AtCl: AtatClient,
    RST: OutputPin,
{
    type Error = Error;

    // Only return a SocketHandle to reference into the SocketSet owned by the UbloxClient,
    // as the Socket object itself provides no value without accessing it though the client.
    type TcpSocket = SocketHandle;

    /// Open a new TCP socket to the given address and port. The socket starts in the unconnected state.
    fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
        self.connected_to_network().map_err(|_| Error::Illegal)?;
        if let Some(ref mut sockets) = self.sockets {
            // Check if there are any unused sockets available
            if sockets.len() >= sockets.capacity() {
                // Check if there are any sockets closed by remote, and close it
                // if it has exceeded its timeout, in order to recycle it.
                if !sockets.recycle() {
                    return Err(Error::SocketSetFull);
                }
            }

            defmt::debug!("[TCP] Opening socket");

            let socket_id = new_socket_num(sockets).unwrap();
            sockets.add(TcpSocket::new(socket_id)).map_err(|e| {
                defmt::error!("[TCP] Opening socket Error: {:?}", e);
                e
            })
        } else {
            Err(Error::Illegal)
        }
    }

    /// Connect to the given remote host and port.
    fn connect(
        &mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        if self.sockets.is_none() {
            return Err(Error::Illegal.into());
        }

        defmt::debug!("[TCP] Connect socket");
        self.connected_to_network().map_err(|_| Error::Illegal)?;

        let url = if let Some(hostname) = self.dns_table.reverse_lookup(remote.ip()) {
            PeerUrlBuilder::new()
                .hostname(hostname.as_str())
                .port(remote.port())
                .creds(self.security_credentials.clone())
                .tcp()
                .map_err(|_| Error::Unaddressable)?
        } else {
            PeerUrlBuilder::new()
                .ip_addr(remote.ip())
                .port(remote.port())
                .creds(self.security_credentials.clone())
                .tcp()
                .map_err(|_| Error::Unaddressable)?
        };

        defmt::debug!("[TCP] Connecting socket: {:?} to url: {=str}", socket, url);

        // If no socket is found we stop here
        let mut tcp = self
            .sockets
            .as_mut()
            .unwrap()
            .get::<TcpSocket<L>>(*socket)
            .map_err(Self::Error::from)?;

        tcp.set_state(TcpState::WaitingForConnect(remote));

        match self
            .send_internal(&EdmAtCmdWrapper(ConnectPeer { url: &url }), false)
            .map_err(|_| Error::Unaddressable)
        {
            Ok(resp) => self
                .socket_map
                .insert_peer(resp.peer_handle, *socket)
                .map_err(|_| Error::InvalidSocket)?,
            Err(e) => {
                let mut tcp = self
                    .sockets
                    .as_mut()
                    .unwrap()
                    .get::<TcpSocket<L>>(*socket)
                    .map_err(Self::Error::from)?;
                tcp.set_state(TcpState::Created);
                return Err(nb::Error::Other(e));
            }
        }

        defmt::debug!("[TCP] Connecting socket: {:?} to url: {=str}", socket, url);

        // TODO: Timeout?
        // TODO: Fix the fact that it doesen't wait for both connect messages
        while {
            matches!(
                self.sockets
                    .as_mut()
                    .unwrap()
                    .get::<TcpSocket<L>>(*socket)
                    .map_err(Self::Error::from)?
                    .state(),
                TcpState::WaitingForConnect(_)
            )
        } {
            self.spin().map_err(|_| Error::Illegal)?;
        }
        Ok(())
    }

    /// Check if this socket is still connected
    fn is_connected(&mut self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        if self.connected_to_network().is_err() {
            return Ok(false);
        }
        if let Some(ref mut sockets) = self.sockets {
            let tcp = sockets.get::<TcpSocket<L>>(*socket)?;
            Ok(tcp.is_connected())
        } else {
            Err(Error::Illegal)
        }
    }

    /// Write to the stream. Returns the number of bytes written is returned
    /// (which may be less than `buffer.len()`), or an error.
    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> nb::Result<usize, Self::Error> {
        self.connected_to_network().map_err(|_| Error::Illegal)?;
        if let Some(ref mut sockets) = self.sockets {
            let tcp = sockets
                .get::<TcpSocket<L>>(*socket)
                .map_err(nb::Error::Other)?;

            if !tcp.is_connected() {
                return Err(Error::SocketClosed.into());
            }

            let channel = *self
                .socket_map
                .socket_to_channel_id(socket)
                .ok_or(nb::Error::Other(Error::SocketClosed))?;

            for chunk in buffer.chunks(EGRESS_CHUNK_SIZE) {
                self.send_internal(
                    &EdmDataCommand {
                        channel,
                        data: chunk,
                    },
                    true,
                )
                .map_err(|_| nb::Error::Other(Error::Unaddressable))?;
            }
            Ok(buffer.len())
        } else {
            Err(Error::Illegal.into())
        }
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        // TODO: Handle error states
        self.spin().map_err(|_| nb::Error::Other(Error::Illegal))?;
        if let Some(ref mut sockets) = self.sockets {
            // Enable detecting closed socket from receive function
            sockets.recycle();

            let mut tcp = sockets
                .get::<TcpSocket<L>>(*socket)
                .map_err(Self::Error::from)?;

            Ok(tcp.recv_slice(buffer).map_err(Self::Error::from)?)
        } else {
            Err(Error::Illegal.into())
        }
    }

    /// Close an existing TCP socket.
    fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        if let Some(ref mut sockets) = self.sockets {
            defmt::debug!("[TCP] Closing socket: {:?}", socket);
            // If the socket is not found it is already removed
            if let Ok(ref tcp) = sockets.get::<TcpSocket<L>>(socket) {
                // If socket is not closed that means a connection excists which has to be closed
                if !matches!(
                    tcp.state(),
                    TcpState::ShutdownForWrite(_) | TcpState::Created
                ) {
                    if let Some(peer_handle) = self.socket_map.socket_to_peer(&tcp.handle()) {
                        let peer_handle = *peer_handle;
                        match self.send_at(ClosePeerConnection { peer_handle }) {
                            Err(crate::error::Error::AT(atat::Error::InvalidResponse)) | Ok(_) => {
                                ()
                            }
                            Err(_) => return Err(Error::Unaddressable),
                        }
                    } else {
                        defmt::error!(
                            "Illigal state! Socket connected but not in socket map: {:?}",
                            tcp.handle()
                        );
                        return Err(Error::Illegal);
                    }
                } else {
                    // No connection exists the socket should be removed from the set here
                    sockets.remove(socket)?;
                }
            }
            Ok(())
        } else {
            Err(Error::Illegal)
        }
    }
}
