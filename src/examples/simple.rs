//! # Simple Example
//!
//! Here is very simple example how to use AgentAI crate. Here we only initialize
//! LLM client, and later use it to answer on simple question.
//!
//! Remember to provide return type for answer, it is very important so library
//! will knows do you expect structured output or just simple String.
//!
//! To run this example from terminal just enter:
//! ```bash
//! cargo run --example simple
//! ```
//!
//! ## Source Code
//!
//! ```rust
#![doc = include_str!("../../examples/simple.rs")]
//! ```
