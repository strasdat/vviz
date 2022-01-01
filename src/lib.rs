//! Rapid prototyping GUI, and visual printf-style debugging for computer vision development.

#![warn(missing_docs)]

pub mod app;
pub mod common;
pub mod entities;
pub mod gui;
pub mod manager;
pub mod math;

// Makes sure that example code in the readme compiles.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
