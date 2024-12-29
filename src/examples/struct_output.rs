//! # Structured Output Example
//!
//! This example introduces concept of structured output supported by most of LLMs. Structured
//! outputs may not work on all LLMs!
//!
//! To achieve it you simply need to create new deserializable structure which will be holding
//! answer. Such structure need to derive from `serde::Deserialize` and `schemars::JsonSchema`.
//!
//! To run this example from terminal just enter:
//! ```bash
//! cargo run --example struct_output
//! ```
//!
//! ## Source Code
//!
//! ```rust
#![doc = include_str!("../../examples/struct_output.rs")]
//! ```
