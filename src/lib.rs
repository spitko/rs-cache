//! Utilities for RuneScape cache interaction.
//! 
//! # Features
//! 
//! The following features are currently provided:
//! - Reading from the cache.
//! - Huffman buffer access.
//! - Checksum with simple-to-use validation.
//! - Compression and decompression:
//!   - Gzip
//!   - Bzip2
//! 
//! # Quick Start
//! 
//! A possible use case of this utility is to read data from the RuneScape cache and send them to the 
//! client during the update protocol.
//! The example listed below quickly shows how you can pass the `index_id` and the `archive_id` to the cache
//! and get the correct data to send to the client.
//! 
//! ```rust
//! # use std::net::TcpStream;
//! # use std::io::Write;
//! use rscache::{ Cache, CacheError, LinkedListExt };
//! # struct UpdatePacket { 
//! #   pub index_id: u8,
//! #   pub archive_id: u8
//! # }
//! 
//! fn process_update(packet: UpdatePacket, stream: &mut TcpStream) -> Result<(), CacheError> {
//! #    let cache = Cache::new("path/to/cache")?;
//!     // read the specified archive from the given index to an owned vector.
//!     let buffer = cache.read(packet.index_id, packet.archive_id)?.to_vec();
//!     
//!     // ... format buffer.
//! #    let formatted_buffer = buffer;
//! 
//!     // send formatted_buffer to client.
//!     stream.write_all(&formatted_buffer)?;
//! 
//!     Ok(())
//! }
//! ```
//! 
//! In the above example the data that was read from the cache is transformed into a vector of bytes.
//! You can also use the `LinkedList<&[u8]>` to `iter()` over the `data_block`s instead of making the bytes owned.
//! 
//! ```rust
//! # use std::net::TcpStream;
//! # use std::io::Write;
//! # use rscache::{ Cache, CacheError, LinkedListExt };
//! # struct UpdatePacket { 
//! #   pub index_id: u8,
//! #   pub archive_id: u8
//! # }
//! # fn process_update(packet: UpdatePacket, stream: &mut TcpStream) -> Result<(), CacheError> {
//! #    let cache = Cache::new("path/to/cache")?;
//! #    // read the specified archive from the given index to an owned vector.
//! let buffer = cache.read(packet.index_id, packet.archive_id)?;
//! 
//! for data_block in buffer.iter() {
//!     // data_block contains 512 byte slices that directly link into the MainData buffer.
//!     // this can be useful when creating a new formatted buffer.
//! }
//! #    Ok(())
//! # }
//! ```

#![warn(clippy::all, clippy::nursery, clippy::clone_on_ref_ptr, clippy::redundant_clone, clippy::default_trait_access, clippy::expl_impl_clone_on_copy,
    clippy::explicit_into_iter_loop, clippy::explicit_iter_loop, clippy::filter_map, clippy::filter_map_next, clippy::find_map, clippy::get_unwrap,
    clippy::items_after_statements, clippy::large_digit_groups, clippy::map_flatten, clippy::match_same_arms, clippy::maybe_infinite_iter, clippy::mem_forget,
    clippy::missing_inline_in_public_items, clippy::multiple_inherent_impl, clippy::mut_mut, clippy::needless_continue,
    clippy::needless_pass_by_value, clippy::option_map_unwrap_or, clippy::option_map_unwrap_or_else, clippy::pub_enum_variant_names, clippy::unused_self,
    clippy::result_map_unwrap_or_else, clippy::similar_names, clippy::single_match_else, clippy::too_many_lines, clippy::type_repetition_in_bounds,
    clippy::unseparated_literal_suffix, clippy::used_underscore_binding)]

#![allow(clippy::suspicious_else_formatting)]

mod cache;
mod checksum;
mod errors;
mod container;
mod traits;

pub use cache::Cache;
pub use checksum::Checksum;
pub use errors::*;
pub use traits::*;
pub use container::{ Container, CompressionType };