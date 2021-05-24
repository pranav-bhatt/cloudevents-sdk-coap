use crate::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use coap_lite::{CoapOption, CoapRequest, Packet};
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::net::SocketAddr;
use std::str;

/// Wrapper for [`CoapRequest`] that implements [`MessageDeserializer`] trait.
pub struct CoapRequestDeserializer {
    pub(crate) options: HashMap<usize, Vec<u8>>,
    pub(crate) payload: Option<Vec<u8>>,
}

impl CoapRequestDeserializer {
    fn get_coap_options(packet: &Packet) -> Result<HashMap<usize, Vec<u8>>> {
        let mut hm = HashMap::new();
        let options = packet.options();

        for i in options {
            let option = (i.0.clone(), i.1.iter().next().unwrap().clone());
            hm.insert(option.0, option.1);
        }
        Ok(hm)
    }

    pub fn new(packet: &Packet) -> Result<CoapRequestDeserializer> {
        Ok(CoapRequestDeserializer {
            options: Self::get_coap_options(packet)?,
            payload: Some(packet.payload.clone()),
        })
    }
}

impl BinaryDeserializer for CoapRequestDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(mut self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            str::from_utf8(
                &self
                    .options
                    .remove(&headers::SPEC_VERSION_OPTION.into())
                    .unwrap(),
            )
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        if let Some(hv) = self.options.remove(&CoapOption::ContentFormat.into()) {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                    cloudevents::message::Error::Other {
                        source: Box::new(e),
                    }
                })?),
            )?
        }

        let mut temp: String;

        for (hn, hv) in self.options.into_iter().filter(|(hn, _)| {
            headers::SPEC_VERSION_OPTION != CoapOption::from(*hn) && *hn >= 2048
            // The first allocation of custom CoAP Option numbers (CE Core + extensions)
        }) {
            let name: &str;
            match headers::OPTIONS_TO_ATTRIBUTES.get(&hn) {
                Some(value) => name = value,
                None => {
                    temp = hn.to_string();
                    name = temp.as_str()
                }
            }

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        cloudevents::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        cloudevents::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            }
        }

        if self.payload != None {
            visitor.end_with_data(self.payload.unwrap())
        } else {
            visitor.end()
        }
    }
}

impl StructuredDeserializer for CoapRequestDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.payload.unwrap())
    }
}

impl MessageDeserializer for CoapRequestDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            self.options
                .get(&CoapOption::ContentFormat.into())
                .map(|s| String::from_utf8(s.to_vec()).ok())
                .flatten()
                .map(|s| s.starts_with(headers::CLOUDEVENTS_JSON_HEADER))
                .unwrap_or(false),
            self.options.get(&headers::SPEC_VERSION_OPTION.into()),
        ) {
            (true, _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform a [`Message`] to [`Event`].
pub fn request_to_event(request: &CoapRequest<SocketAddr>) -> Result<Event> {
    MessageDeserializer::into_event(CoapRequestDeserializer::new(&request.message)?)
}

/// Extension Trait for [`Message`] which acts as a wrapper for the function [`record_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait MessageExt: private::Sealed {
    /// Generates [`Event`] from [`BorrowedMessage`].
    fn to_event(&self) -> Result<Event>;
}

impl MessageExt for CoapRequest<SocketAddr> {
    fn to_event(&self) -> Result<Event> {
        request_to_event(self)
    }
}

mod private {
    // Sealing the MessageExt
    pub trait Sealed {}
    impl Sealed for coap_lite::CoapRequest<std::net::SocketAddr> {}
}
