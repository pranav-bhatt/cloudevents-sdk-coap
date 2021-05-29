use coap_lite::CoapOption;
use lazy_static::lazy_static;
use std::collections::HashMap;

fn options_to_attributes() -> HashMap<usize, &'static str> {
    let mut op_to_attr = HashMap::new();
    op_to_attr.insert(4200, "id");
    op_to_attr.insert(4201, "source");
    op_to_attr.insert(4202, "specversion"); // TODO redundant. Come back to this later
    op_to_attr.insert(4203, "type");
    op_to_attr.insert(12, "datacontenttype");
    op_to_attr.insert(4204, "dataschema");
    op_to_attr.insert(4205, "subject");
    op_to_attr.insert(4206, "time");
    op_to_attr
}

fn attributes_to_options() -> HashMap<&'static str, CoapOption> {
    let mut attr_to_op = HashMap::new();
    attr_to_op.insert("id", CoapOption::Unknown(4200));
    attr_to_op.insert("source", CoapOption::Unknown(4201));
    attr_to_op.insert("specversion", CoapOption::Unknown(4202)); // TODO redundant. Come back to this later
    attr_to_op.insert("type", CoapOption::Unknown(4203));
    attr_to_op.insert("datacontenttype", CoapOption::ContentFormat);
    attr_to_op.insert("dataschema", CoapOption::Unknown(4204));
    attr_to_op.insert("subject", CoapOption::UriPath);
    attr_to_op.insert("time", CoapOption::Unknown(4206));
    attr_to_op
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_OPTIONS: HashMap<&'static str, CoapOption> =
        attributes_to_options();
    pub(crate) static ref OPTIONS_TO_ATTRIBUTES: HashMap<usize, &'static str> =
        options_to_attributes();
}

pub(crate) static SPEC_VERSION_OPTION: CoapOption = CoapOption::Unknown(4202);
pub(crate) static CONTENT_TYPE: CoapOption = CoapOption::ContentFormat;
pub(crate) static CLOUDEVENTS_JSON_HEADER: &'static str = "application/cloudevents+json";

// Drogue-IoT extention attribute: `auth-token`
// pub(crate) static AUTH_TOKEN_OPTION: CoapOption = CoapOption::Unknown(2055);
