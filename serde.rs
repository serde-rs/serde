#![feature(macro_rules, phase)]

extern crate collections;

// test harness access
#[cfg(test)]
extern crate test;
#[phase(syntax, link)]
extern crate log;

pub mod de;
//pub mod json;
