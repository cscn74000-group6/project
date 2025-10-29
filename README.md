# CSCN74000 Project 🦀
This is a flight collision detection system for the CSCN74000 Software Safety and Reliability course.

## Goal
The goal of this project is to write a mission critical aviation system using Rust. This is an experiment to see how Rust compares to MISRA-C style code.  

## MIS-RUST:2025
We have adopted several coding rules inspired by MISRA.
1. The unsafe keyword is banned.
2. Use mutable variables.
   1. Unless _**absolutely**_ necessary. Exceptions include a TCP Stream or state machine flag.
4. No panics.
5. All functions must return a Result type.
6. All Result types must be explicitly checked.

## Setup
1. Clone project.
2. Run with cargo.
