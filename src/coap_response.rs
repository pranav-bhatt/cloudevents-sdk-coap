use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use coap_lite::Packet;

/// This struct contains a serialized CloudEvent message in the Kafka shape.
/// Implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
///
/// To instantiate a new `MessageRecord` from an [`Event`],
/// look at [`Self::from_event`] or use [`StructuredDeserializer::deserialize_structured`](cloudevents::message::StructuredDeserializer::deserialize_structured)
/// or [`BinaryDeserializer::deserialize_binary`].
pub struct CoapMessage {
    pub(crate) packet: Packet  
}

impl CoapMessage {
    /// Create a new empty [`MessageRecord`]
    pub fn new() -> Self {
        CoapMessage {
            packet: Packet::new()
        }
    }

    /// Create a new [`MessageRecord`], filled with `event` serialized in binary mode.
    pub fn from_event(event: Event) -> Result<Self> {
        BinaryDeserializer::deserialize_binary(event, CoapMessage::new())
    }
}

impl Default for CoapMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySerializer<CoapMessage> for CoapMessage {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self
            .packet
            .add_option(headers::SPEC_VERSION_OPTION, spec_version.as_str().as_bytes().to_vec());

        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.packet.add_option(
            *headers::ATTRIBUTES_TO_OPTIONS
                .get(name)
                .ok_or(cloudevents::message::Error::UnknownAttribute {
                    name: String::from(name),
                })?,
            value.to_string()[..].as_bytes().to_vec(),
        );

        Ok(self)
    }

    
    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        /*
        self.packet = self
            .packet
            .add_option(&attribute_name_to_header!(name)[..], &value.to_string()[..]);
        */
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<CoapMessage> {
        self.packet.payload = bytes;

        Ok(self)
    }

    fn end(self) -> Result<CoapMessage> {
        Ok(self)
    }
}

impl StructuredSerializer<CoapMessage> for CoapMessage {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<CoapMessage> {
        self
        .packet
        .add_option(headers::CONTENT_TYPE, headers::CLOUDEVENTS_JSON_HEADER.as_bytes().to_vec());

        self.packet.payload = bytes;

        Ok(self)
    }
}

/// Extension Trait for [`BaseRecord`] that fills the record with a [`MessageRecord`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait CoapMessageExt<'a>: private::Sealed {
    /// Fill this [`BaseRecord`] with a [`MessageRecord`].
    fn coap_message(
        self,
        coap_message: &'a Packet,
    ) -> Result<Packet>;
}

impl<'a> CoapMessageExt<'a> for Packet {
    fn coap_message(
        mut self,
        coap_message: &'a Packet,
    ) -> Result<Packet> {
        self = coap_message.clone();

        Ok(self)
    }
}

mod private {
    // Sealing the FutureRecordExt and BaseRecordExt
    pub trait Sealed {}
    
    impl Sealed for coap_lite::Packet {}
}

