use core::cmp::min;

use super::{ChannelId, Error, Result, RingBuffer, Socket, SocketHandle, SocketMeta};
use core::convert::TryInto;
use embedded_nal::{Ipv4Addr, SocketAddr, SocketAddrV4};
use embedded_time::duration::{Generic, Milliseconds, Seconds};
use embedded_time::{Clock, Instant};

/// A UDP socket ring buffer.
pub type SocketBuffer<const N: usize> = RingBuffer<u8, N>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Closed,
    Established,
}

impl Default for State {
    fn default() -> Self {
        State::Closed
    }
}

/// A User Datagram Protocol socket.
///
/// A UDP socket is bound to a specific endpoint, and owns transmit and receive
/// packet buffers.
pub struct UdpSocket<CLK: Clock, const L: usize> {
    pub(crate) meta: SocketMeta,
    pub(crate) endpoint: SocketAddr,
    _available_data: usize,
    read_timeout: Option<Seconds>,
    state: State,
    rx_buffer: SocketBuffer<L>,
    closed_time: Option<Instant<CLK>>,
}

impl<CLK: Clock, const L: usize> UdpSocket<CLK, L> {
    /// Create an UDP socket with the given buffers.
    pub fn new(socket_id: usize) -> Self {
        let mut meta = SocketMeta::default();
        meta.handle.0 = socket_id;
        UdpSocket {
            meta,
            endpoint: SocketAddrV4::new(Ipv4Addr::unspecified(), 0).into(),
            state: State::Closed,
            _available_data: 0,
            read_timeout: Some(Seconds(15)),
            rx_buffer: SocketBuffer::new(),
            closed_time: None,
        }
    }

    /// Return the socket handle.
    #[inline]
    pub fn handle(&self) -> SocketHandle {
        self.meta.handle
    }

    /// Return the socket channel id.
    #[inline]
    pub fn channel_id(&self) -> ChannelId {
        self.meta.channel_id
    }

    /// Return the bound endpoint.
    #[inline]
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    /// Return the connection state, in terms of the UDP connection.
    #[inline]
    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    pub fn recycle(&self, ts: &Instant<CLK>) -> bool
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        if let Some(read_timeout) = self.read_timeout {
            self.closed_time
                .and_then(|ref closed_time| ts.checked_duration_since(closed_time))
                .and_then(|dur| dur.try_into().ok())
                .map(|dur: Milliseconds<u32>| dur >= read_timeout)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Bind the socket to the given endpoint.
    ///
    /// This function returns `Err(Error::Illegal)` if the socket was open
    /// (see [is_open](#method.is_open)), and `Err(Error::Unaddressable)`
    /// if the port in the given endpoint is zero.
    pub fn bind<U: Into<SocketAddr>>(&mut self, endpoint: U) -> Result<()> {
        if self.is_open() {
            return Err(Error::Illegal);
        }

        let endpoint = endpoint.into();
        match endpoint {
            SocketAddr::V4(ipv4) => {
                if ipv4.port() == 0 {
                    return Err(Error::Unaddressable);
                }
            }
            SocketAddr::V6(ipv6) => {
                if ipv6.port() == 0 {
                    return Err(Error::Unaddressable);
                }
            }
        }

        self.endpoint = endpoint;
        Ok(())
    }

    /// Check whether the socket is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        match self.state {
            State::Established => true,
            _ => false,
        }
    }

    /// Check whether the receive buffer is full.
    #[inline]
    pub fn can_recv(&self) -> bool {
        !self.rx_buffer.is_full()
    }

    // /// Return the maximum number packets the socket can receive.
    // #[inline]
    // pub fn packet_recv_capacity(&self) -> usize {
    //     self.rx_buffer.packet_capacity()
    // }

    // /// Return the maximum number of bytes inside the recv buffer.
    // #[inline]
    // pub fn payload_recv_capacity(&self) -> usize {
    //     self.rx_buffer.payload_capacity()
    // }

    fn recv_impl<'b, F, R>(&'b mut self, f: F) -> Result<R>
    where
        F: FnOnce(&'b mut SocketBuffer<L>) -> (usize, R),
    {
        // We may have received some data inside the initial SYN, but until the connection
        // is fully open we must not dequeue any data, as it may be overwritten by e.g.
        // another (stale) SYN. (We do not support TCP Fast Open.)
        if !self.is_open() {
            return Err(Error::Illegal);
        }

        let (_size, result) = f(&mut self.rx_buffer);
        Ok(result)
    }

    /// Dequeue a packet received from a remote endpoint, and return the endpoint as well
    /// as a pointer to the payload.
    ///
    /// This function returns `Err(Error::Exhausted)` if the receive buffer is empty.
    pub fn recv<'b, F, R>(&'b mut self, f: F) -> Result<R>
    where
        F: FnOnce(&'b mut [u8]) -> (usize, R),
    {
        self.recv_impl(|rx_buffer| rx_buffer.dequeue_many_with(f))
    }

    /// Dequeue a packet received from a remote endpoint, copy the payload into the given slice,
    /// and return the amount of octets copied as well as the endpoint.
    ///
    /// See also [recv](#method.recv).
    pub fn recv_slice(&mut self, data: &mut [u8]) -> Result<usize> {
        self.recv_impl(|rx_buffer| {
            let size = rx_buffer.dequeue_slice(data);
            (size, size)
        })
    }

    pub fn rx_enqueue_slice(&mut self, data: &[u8]) -> usize {
        self.rx_buffer.enqueue_slice(data)
    }

    /// Peek at a packet received from a remote endpoint, and return the endpoint as well
    /// as a pointer to the payload without removing the packet from the receive buffer.
    /// This function otherwise behaves identically to [recv](#method.recv).
    ///
    /// It returns `Err(Error::Exhausted)` if the receive buffer is empty.
    pub fn peek(&mut self, size: usize) -> Result<&[u8]> {
        if !self.is_open() {
            return Err(Error::Illegal);
        }

        Ok(self.rx_buffer.get_allocated(0, size))
    }

    /// Peek at a packet received from a remote endpoint, copy the payload into the given slice,
    /// and return the amount of octets copied as well as the endpoint without removing the
    /// packet from the receive buffer.
    /// This function otherwise behaves identically to [recv_slice](#method.recv_slice).
    ///
    /// See also [peek](#method.peek).
    pub fn peek_slice(&mut self, data: &mut [u8]) -> Result<usize> {
        let buffer = self.peek(data.len())?;
        let length = min(data.len(), buffer.len());
        data[..length].copy_from_slice(&buffer[..length]);
        Ok(length)
    }

    pub fn close(&mut self) {
        self.endpoint.set_port(0);
    }
}

impl<CLK: Clock, const L: usize> Into<Socket<CLK, L>> for UdpSocket<CLK, L> {
    fn into(self) -> Socket<CLK, L> {
        Socket::Udp(self)
    }
}
