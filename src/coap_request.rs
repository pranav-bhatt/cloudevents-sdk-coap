use crate::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use coap_lite::{CoapOption, Packet};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

/// Wrapper for [`CoapRequest`] that implements [`MessageDeserializer`] trait.
pub struct CoapRequestDeserializer {
    pub(crate) options: HashMap<usize, String>,
    pub(crate) payload: Option<Vec<u8>>,
}

impl CoapRequestDeserializer {
    fn get_coap_options(packet: &Packet) -> Result<HashMap<usize, String>> {
        let mut hm = HashMap::new();
        let options = packet.options();

        let mut hv = String::new();

        for i in options {
            // Drogue IoT specific
            match CoapOption::from(*i.0) {
                CoapOption::UriPath => {
                    for j in i.1.clone() {
                        hv.push_str(str::from_utf8(&j).map_err(|e| {
                            cloudevents::message::Error::Other {
                                source: Box::new(e),
                            }
                        })?);
                        hv.push('/');
                    }
                    // Removing trailing '/'
                    hv.pop();
                }
                _ => {
                    hv = String::from_utf8(i.1.back().unwrap().clone()).map_err(|e| {
                        cloudevents::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?
                }
            }
            hm.insert(*i.0, hv.clone());
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
            &self
                .options
                .remove(&headers::SPEC_VERSION_OPTION.into())
                .unwrap()[..],
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        if let Some(hv) = self.options.remove(&CoapOption::ContentFormat.into()) {
            visitor = visitor.set_attribute("datacontenttype", MessageAttributeValue::String(hv))?
        }

        let mut temp: String;

        for (hn, hv) in self
            .options
            .into_iter()
            .filter(|(hn, _)| *hn >= 2048 || *hn == 11)
        {
            // CoAP Option 11: Uri-Path
            // The first allocation of custom CoAP Option numbers (CE Core + extensions)
            let name: &str;
            match headers::OPTIONS_TO_ATTRIBUTES.get(&hn) {
                Some(value) => name = value,
                None => {
                    temp = hn.to_string();
                    name = temp.as_str()
                }
            }

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(name, MessageAttributeValue::String(hv))?
            } else {
                visitor = visitor.set_extension(name, MessageAttributeValue::String(hv))?
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
                .unwrap()
                .starts_with(headers::CLOUDEVENTS_JSON_HEADER),
            self.options.get(&headers::SPEC_VERSION_OPTION.into()),
        ) {
            (true, _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}
