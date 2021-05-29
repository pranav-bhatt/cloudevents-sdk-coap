use cloudevents::message::{MessageDeserializer, Result};
use cloudevents::Event;
use coap_lite::CoapRequest;
use std::net::SocketAddr;

pub mod coap_request;
pub mod coap_response;
pub mod headers;

use coap_request::CoapRequestDeserializer;
use coap_response::CoapMessage;

/// Extension Trait for [`CoapRequest`] which implements two additional functions: `].
/// The [`to_event()`] function deserializes a [`CoapRequest<SocketAddr>`] into an [`Event`],
/// and the [`from_event()`] creates a new [`CoapRequest<SocketAddr>`] and fills it with an [`Event`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait CoapRequestExt: private::Sealed {
    /// Generates [`Event`] from [`CoapRequest`].
    fn to_event(&self) -> Result<Event>;
    /// Generates [`CoapRequest`] from [`Event`].
    fn from_event(event: Event) -> Result<CoapRequest<SocketAddr>>;
}

impl CoapRequestExt for CoapRequest<SocketAddr> {
    fn to_event(&self) -> Result<Event> {
        MessageDeserializer::into_event(CoapRequestDeserializer::new(&self.message)?)
    }
    fn from_event(event: Event) -> Result<CoapRequest<SocketAddr>> {
        Ok(CoapMessage::from_event(event).unwrap().coap_message)
    }
}

mod private {
    // Sealing the MessageExt
    pub trait Sealed {}
    impl Sealed for coap_lite::CoapRequest<std::net::SocketAddr> {}
}
