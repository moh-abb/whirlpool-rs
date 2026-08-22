pub mod amy_adapter;
pub mod scheduler;
pub mod unit;

#[allow(unused)]
#[allow(unnecessary_transmutes)]
#[allow(unused_qualifications)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
mod amy_bindings {
    include!(concat!(env!("OUT_DIR"), "/amy_bindings.rs"));
}
