//! This library implements a series of parsers for the data returned by the binance
//! "/eapi/v1/ticker" GET endpoint.
//!
//! The data consists of a JSON array containing N JSON Objects. Each JSON object contains 18
//! fields that have either:
//!  - floating point numbers encoded as JSON strings
//!  - integer numbers encoded as JSON numbers.
//!  - regular JSON strings.
//!
//! Each module implements a different parsing strategy with different tradeoffs reducing latency.

/// This is a simple parser of the data, which uses the serde and serde_json libraries.
/// Serde is actually reasonably well optimized and does a great job at parsing JSON.
///
/// Downsides of this parser include:
///  - It converts every single value inside the JSON object to its represented type. This incurs
///    in potentially more work than actually needed if we are only interested in a subset of the
///    fields of the object.
///  - It parses every single entry of a JSON Array and every single field in a JSON object,
///    representing the full JSON message in memory. If we were only interested in the first 10
///    entries of the array, this parser would not be a good fit.
///  - Allocates a `String` for the `symbol` field.
///
/// Upsides:
///  - Simple and easy to understand.
///  - If you actually need all the data transformed to a format that is suitable for user
///    consuptions, this parser does a good job.
///  - You don't need to keep the original message around, as the parsed representation is
///    fully-owned.
pub mod serde;

/// Also a simple parser, but represents all the data inside the JSON as borrowed strings.
///
/// Downsides of this parser include:
///  - The user still needs to convert each field to their actual value (e.g. f64 or u64).
///  - The information about the type of each file is not encoded in the result, making it more
///    error-prone.
///  - It parses every single entry of a JSON Array and every single field in a JSON object,
///    representing the full JSON message in memory. If we were only interested in the first 10
///    entries of the array, this parser would not be a good fit.
///  - You need to keep the original message around, as the parsed representation is borrowed from
///    the message.
///
/// Upsides:
///  - Only does allocations for the Vec<> containing the (compared to the `serde` module).
///  - Simple and easy to understand.
///  - Faster than the `serde` parser, because it does not perform that much work ahead of time.
pub mod serde_borrowed;

/// An improvement over `serde_borrowed` that wraps the `&str` members using a wrapper type that
/// allows to lazily and on-demand convert a `&str` to the target type (say `f64`).
///
/// Downsides of this parser include:
///  - It parses every single entry of a JSON Array and every single field in a JSON object,
///    representing the full JSON message in memory. If we were only interested in the first 10
///    entries of the array, this parser would not be a good fit.
///  - You need to keep the original message around, as the parsed representation is borrowed from
///    the message.
///
/// Upsides:
///  - Only does allocations for the Vec<> containing the (compared to the `serde` module).
///  - Simple and easy to understand.
///  - Faster than the `serde` parser, because it does not perform that much work ahead of time.
///  - More ergonomic than `serde_borrowed` because it keeps the type information of each field.
pub mod serde_lazy;

/// An improvement over `serde_lazy` that uses the sonic-rs library instead of serde.
/// This library benefits from SIMD operations during parsing. Apart from this, it implements the
/// same solution as `serde_lazy`.
///
/// Downsides of this parser include:
///  - It parses every single entry of a JSON Array and every single field in a JSON object,
///    representing the full JSON message in memory. If we were only interested in the first 10
///    entries of the array, this parser would not be a good fit.
///  - You need to keep the original message around, as the parsed representation is borrowed from
///    the message.
///
/// Upsides:
///  - Only does allocations for the Vec<> containing the (compared to the `serde` module).
///  - Simple and easy to understand.
///  - Faster than the `serde` parser, because it does not perform that much work ahead of time.
///  - More ergonomic than `serde_borrowed` because it keeps the type information of each field.
///  - Faster than serde because of the use of SIMD.
pub mod sonic;

/// Implements a custom hand-crafted JSON parser that constructs a JSON AST in memory.
/// I reused most of this code from a personal project (git-dashboard), which is not yet publicly
/// available.
/// This parser performs (unsurprisingly) poorly when compared to more mature solutions like serde
/// or sonic-rs.
///
/// The complexity of this parser scales linearly with `N`, but the amount of work it performs to
/// parse a single entry is reasonably large, making it not very suitable for a low-latency
/// environment.
///
/// Possible improvements:
///  - Parse the JSON lazily based on the data actually needed by the user.
///  - Use faster parsing using vectorization.
///  - Keep less metadata on each value.
///  - Reduce the number of allocations needed to represent a value.
pub mod custom;

/// Implements a custom hand-crafted lazy parser. This parser implements a different concept than
/// all the `serde-based` parsers.
///
/// Instead of constructing an AST of the JSON message, this parser does not do any work ahead of
/// time. Instead it exposes an API for the user to query the data inside the message.
///
/// This parser may be used as:
/// ```rust
/// # fn main() -> anyhow::Result<()> {
///     let document = binance::custom_lazy::Document::new(&r##"[{"symbol": "test"}]"##);
///     let symbol = document
///         .as_array()?
///         .get_index(0)?
///         .as_object()?
///         .get_key("symbol")?
///         .as_string()?
///         .get_value()?;
/// #   Ok(())
/// # }
/// ```
///
/// In this example, the parser only does any parsing work as requested by the user. In the example
/// above, the call to `get_index` on the array triggers the parser to find the first entry inside
/// the JSON array, but it simply returns an object with a cursor pointing at the element, it does
/// not parse the rest of the data.
///
/// Then, when the user requests a key, it traverses the JSON message until it finds the key and
/// returns an object that points to the value inside the key in the JSON message. Finally, the
/// user requests to get the value as a string, which simply reads the string from the JSON
/// message.
///
/// Note that, with this parser, even if the message had 1 million entries, we only needed to parse
/// part of the first message, which could be a really big advantage.
///
/// This parser is zero-alloc.
///
/// This parser is motivated by <https://arxiv.org/abs/2312.17149>
///
/// Possible improvements:
///  - Make the Array and Object elements remember the last cursor of the entry they last read.
///    This is not currently implemented, so if I asked an array object for the 88th entry and then
///    for the 89th, it would re-parse the array from the beginning, making it rather inefficient.
///  - Use faster parsing using vectorization.
pub mod custom_lazy;

/// Development utilities used by more than 1 parser.
mod utils;
