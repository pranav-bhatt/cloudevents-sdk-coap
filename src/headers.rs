use coap_lite::CoapOption;
use lazy_static::lazy_static;
use std::collections::HashMap;

fn options_to_attributes() -> HashMap<usize, &'static str> {
    let mut op_to_attr = HashMap::new();
    op_to_attr.insert(2048, "id");
    op_to_attr.insert(2049, "source");
    op_to_attr.insert(2050, "specversion"); // TODO redundant. Come back to this later
    op_to_attr.insert(2051, "type");
    op_to_attr.insert(12, "datacontenttype");
    op_to_attr.insert(2052, "dataschema");
    op_to_attr.insert(2053, "subject");
    op_to_attr.insert(2054, "time");
    op_to_attr
}

fn attributes_to_options() -> HashMap<&'static str, CoapOption> {
    let mut attr_to_op = HashMap::new();
    attr_to_op.insert("id", CoapOption::Unknown(2048));
    attr_to_op.insert("source", CoapOption::Unknown(2049));
    attr_to_op.insert("specversion", CoapOption::Unknown(2050)); // TODO redundant. Come back to this later
    attr_to_op.insert("type", CoapOption::Unknown(2051));
    attr_to_op.insert("datacontenttype", CoapOption::ContentFormat);
    attr_to_op.insert("dataschema", CoapOption::Unknown(2052));
    attr_to_op.insert("subject", CoapOption::Unknown(2053));
    attr_to_op.insert("time", CoapOption::Unknown(2054));
    attr_to_op
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_OPTIONS: HashMap<&'static str, CoapOption> =
        attributes_to_options();
    pub(crate) static ref OPTIONS_TO_ATTRIBUTES: HashMap<usize, &'static str> =
        options_to_attributes();
}

pub(crate) static SPEC_VERSION_OPTION: CoapOption = CoapOption::Unknown(2050);
pub(crate) static CONTENT_TYPE: CoapOption = CoapOption::ContentFormat;
pub(crate) static CLOUDEVENTS_JSON_HEADER: &'static str = "application/cloudevents+json";
pub(crate) static CLOUDEVENTS_COAP_MAPPINGS: [usize; 8] =
    [12, 2048, 2049, 2050, 2051, 2052, 2053, 2054];

// Drogue-IoT extention attribute: `auth-token`
// pub(crate) static AUTH_TOKEN_OPTION: CoapOption = CoapOption::Unknown(2055);
