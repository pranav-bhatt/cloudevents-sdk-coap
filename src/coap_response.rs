use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use coap_lite::{CoapOption, CoapRequest};
use std::net::SocketAddr;

/// This struct contains a serialized CloudEvent message in the Kafka shape.
/// Implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
///
/// To instantiate a new `MessageRecord` from an [`Event`],
/// look at [`Self::from_event`] or use [`StructuredDeserializer::deserialize_structured`](cloudevents::message::StructuredDeserializer::deserialize_structured)
/// or [`BinaryDeserializer::deserialize_binary`].
pub struct CoapMessage {
    pub(crate) coap_message: CoapRequest<SocketAddr>,
    options: Vec<u16>,
}

impl CoapMessage {
    /// Create a new empty [`CoapMessage`]
    pub fn new() -> Self {
        CoapMessage {
            coap_message: CoapRequest::new(),
            options: vec![12, 2048, 2049, 2050, 2051, 2052, 2053, 2054],
        }
    }

    /// Create a new [`CoapMessage`], filled with `event` serialized in binary mode.
    pub fn from_event(event: Event) -> Result<Self> {
        BinaryDeserializer::deserialize_binary(event, Self::new())
    }
}

impl Default for CoapMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySerializer<CoapMessage> for CoapMessage {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.coap_message.message.add_option(
            headers::SPEC_VERSION_OPTION,
            spec_version.as_str().as_bytes().to_vec(),
        );

        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        // Drogue IoT specific
        if self
            .coap_message
            .message
            .get_option(CoapOption::ContentFormat)
            == None
        {
            self.coap_message.message.add_option(
                CoapOption::ContentFormat,
                "application/octet-stream".as_bytes().to_vec(),
            );
        }
        if self
            .coap_message
            .message
            .get_option(CoapOption::Unknown(4203))
            == None
        {
            self.coap_message.message.add_option(
                CoapOption::Unknown(4203),
                "io.drogue.event.v1".as_bytes().to_vec(),
            );
        }
        self.coap_message.message.add_option(
            *headers::ATTRIBUTES_TO_OPTIONS.get(name).ok_or(
                cloudevents::message::Error::UnknownAttribute {
                    name: String::from(name),
                },
            )?,
            value.to_string().as_bytes().to_vec(),
        );

        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        // Try to obtain usize from &str
        let option_number = name.parse::<u16>().unwrap();
        // If option type is reserved by the spec, or has already been set
        // (convert LinkedList to bytes and store for multiple values per option)
        if self.options.contains(&option_number) {
            return Err(cloudevents::message::Error::UnknownAttribute {
                name: String::from(format!(
                    "CoAP Option number is either restricted or has already been set: {}",
                    name
                )), // TODO The error type is not correct
            });
        }
        // insert new extension option into restriction list
        match self.options.binary_search(&option_number) {
            Ok(_pos) => {}
            Err(pos) => self.options.insert(pos, option_number),
        }
        // Add option to CoapRequest message
        self.coap_message.message.add_option(
            CoapOption::from(option_number),
            value.to_string().as_bytes().to_vec(),
        );

        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<CoapMessage> {
        self.coap_message.message.payload = bytes;

        Ok(self)
    }

    fn end(self) -> Result<CoapMessage> {
        Ok(self)
    }
}

impl StructuredSerializer<CoapMessage> for CoapMessage {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<CoapMessage> {
        self.coap_message.message.add_option(
            headers::CONTENT_TYPE,
            headers::CLOUDEVENTS_JSON_HEADER.as_bytes().to_vec(),
        );

        self.coap_message.message.payload = bytes;

        Ok(self)
    }
}

/// Extension Trait for [`CoapRequest`] that creates a new CoapRequest and fills it with an [`Event`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait CoapRequestExt: private::Sealed {
    fn from_event(event: Event) -> Result<CoapRequest<SocketAddr>>;
}

impl CoapRequestExt for CoapRequest<SocketAddr> {
    fn from_event(event: Event) -> Result<CoapRequest<SocketAddr>> {
        Ok(CoapMessage::from_event(event).unwrap().coap_message)
    }
}

mod private {
    // Sealing the CoapRequestExt
    pub trait Sealed {}

    impl Sealed for coap_lite::CoapRequest<std::net::SocketAddr> {}
}
