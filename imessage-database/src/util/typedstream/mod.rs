/*!
 Contains logic and data structures used to parse and deserialize `typedstream` data into native Rust data structures.

 ## Overview

 The typedstream format is a binary serialization protocol designed for `C` and `Objective-C` data structures.
 It is primarily used in Apple's Foundation framework, specifically within the `NSArchiver` and `NSUnarchiver` classes.

 ## Origin

 The format is derived from the data structure used by `NeXTSTEP`'s `NXTypedStream` APIs.

 ## Features

 - Pure Rust implementation for efficient and safe deserialization
 - No dependencies on Apple frameworks
 - Robust error handling for malformed or incomplete `typedstream` data

 ## Reverse Engineering

 A blog post describing the reverse engineering of `typedstream` is located [here](https://chrissardegna.com/blog/reverse-engineering-apples-typedstream-format/).
*/

pub mod models;
pub mod parser;
mod tests;
