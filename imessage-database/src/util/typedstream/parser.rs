/*!
 Logic used to deserialize data from a `typedstream`, focussing specifically on [`NSAttributedString`](https://developer.apple.com/documentation/foundation/nsattributedstring).

 Logic reverse engineered from `typedstream` source located at:
   - [`typedstream.h`](https://github.com/gnustep/libobjc/blob/master/objc/typedstream.h)
   - [`archive.c`](https://github.com/gnustep/libobjc/blob/master/archive.c)
   - [`objc/typedstream.m`](https://securitronlinux.com/news/html/d4/d6c/typedstream_8m.html)

 A writeup about the reverse engineering of `typedstream` can be found [here](https://chrissardegna.com/blog/reverse-engineering-apples-typedstream-format/).
*/
use std::collections::HashSet;

use crate::{
    error::typedstream::TypedStreamError,
    util::typedstream::models::{Archivable, Class, ClassResult, OutputData, Type},
};

/// Indicates an [`i16`] in the byte stream
const I_16: u8 = 0x81;
/// Indicates an [`i32`] in the byte stream
const I_32: u8 = 0x82;
/// Indicates an [`f32`] or [`f64`] in the byte stream; the [`Type`] determines the size
const DECIMAL: u8 = 0x83;
/// Indicates the start of a new object
const START: u8 = 0x84;
/// Indicates that there is no more data to parse, for example the end of a class inheritance chain
const EMPTY: u8 = 0x85;
/// Indicates the last byte of an object
const END: u8 = 0x86;
/// Bytes equal or greater in value than the reference tag indicate an index in the table of already-seen types
const REFERENCE_TAG: u64 = 0x92;

/// Contains logic and data used to deserialize data from a `typedstream`.
///
/// `typedstream` is a binary serialization format developed by `NeXT` and later adopted by Apple.
/// It's designed to serialize and deserialize complex object graphs and data structures in C and Objective-C.
///
/// A `typedstream` begins with a header that includes format version and architecture information,
/// followed by a stream of typed data elements. Each element is prefixed with type information,
/// allowing the [`TypedStreamReader`] to understand the original data structures.
#[derive(Debug)]
pub struct TypedStreamReader<'a> {
    /// The `typedstream` we want to parse
    stream: &'a [u8],
    /// The current index we are at in the stream
    idx: usize,
    /// As we parse the `typedstream`, build a table of seen [`Type`]s to reference in the future
    ///
    /// The first time a [`Type`] is seen, it is present in the stream literally,
    /// but afterwards are only referenced by index in order of appearance.
    types_table: Vec<Vec<Type>>,
    /// As we parse the `typedstream`, build a table of seen archivable data to reference in the future
    object_table: Vec<Archivable>,
    /// We want to copy embedded types the first time they are seen, even if the types were resolved through references
    seen_embedded_types: HashSet<u32>,
    /// Stores the position of the current [`Archivable::Placeholder`]
    placeholder: Option<usize>,
}

impl<'a> TypedStreamReader<'a> {
    /// Given a stream, construct a reader instance to parse it.
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::typedstream::parser::TypedStreamReader;
    ///
    /// let bytes: Vec<u8> = vec![]; // Example stream
    /// let mut reader = TypedStreamReader::from(&bytes);
    /// ```
    #[must_use]
    pub fn from(stream: &'a [u8]) -> Self {
        Self {
            stream,
            idx: 0,
            types_table: vec![],
            object_table: vec![],
            seen_embedded_types: HashSet::new(),
            placeholder: None,
        }
    }

    /// Read a signed integer from the stream. Because we don't know the size of the integer ahead of time,
    /// we store it in the largest possible value.
    fn read_signed_int(&mut self) -> Result<i64, TypedStreamError> {
        match self.get_current_byte()? {
            I_16 => {
                let size = 2;
                self.idx += 1;
                let value = i16::from_le_bytes(
                    <[u8; 2]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(i64::from(value))
            }
            I_32 => {
                let size = 4;
                self.idx += 1;
                let value = i32::from_le_bytes(
                    <[u8; 4]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(i64::from(value))
            }
            _ => {
                if self.get_current_byte()? > REFERENCE_TAG as u8 && self.get_next_byte()? != END {
                    self.idx += 1;
                    return self.read_signed_int();
                }
                let value = i8::from_le_bytes([self.get_current_byte()?]);
                self.idx += 1;
                Ok(i64::from(value))
            }
        }
    }

    /// Read an unsigned integer from the stream. Because we don't know the size of the integer ahead of time,
    /// we store it in the largest possible value.
    fn read_unsigned_int(&mut self) -> Result<u64, TypedStreamError> {
        match self.get_current_byte()? {
            I_16 => {
                let size = 2;
                self.idx += 1;
                let value = u16::from_le_bytes(
                    <[u8; 2]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(u64::from(value))
            }
            I_32 => {
                let size = 4;
                self.idx += 1;
                let value = u32::from_le_bytes(
                    <[u8; 4]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(u64::from(value))
            }
            _ => {
                let value = u8::from_le_bytes([self.get_current_byte()?]);
                self.idx += 1;
                Ok(u64::from(value))
            }
        }
    }

    /// Read a single-precision float from the byte stream
    fn read_float(&mut self) -> Result<f32, TypedStreamError> {
        match self.get_current_byte()? {
            DECIMAL => {
                let size = 4;
                self.idx += 1;
                let value = f32::from_le_bytes(
                    <[u8; 4]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(value)
            }
            I_16 | I_32 => Ok(self.read_signed_int()? as f32),
            _ => {
                self.idx += 1;
                Ok(self.read_signed_int()? as f32)
            }
        }
    }

    /// Read a double-precision float from the byte stream
    fn read_double(&mut self) -> Result<f64, TypedStreamError> {
        match self.get_current_byte()? {
            DECIMAL => {
                let size = 8;
                self.idx += 1;
                let value = f64::from_le_bytes(
                    <[u8; 8]>::try_from(self.read_exact_bytes(size)?)
                        .map_err(TypedStreamError::SliceError)?,
                );
                Ok(value)
            }
            I_16 | I_32 => Ok(self.read_signed_int()? as f64),
            _ => {
                self.idx += 1;
                Ok(self.read_signed_int()? as f64)
            }
        }
    }

    /// Read exactly `n` bytes from the stream
    fn read_exact_bytes(&mut self, n: usize) -> Result<&[u8], TypedStreamError> {
        let range =
            self.stream
                .get(self.idx..self.idx + n)
                .ok_or(TypedStreamError::OutOfBounds(
                    self.idx + n,
                    self.stream.len(),
                ))?;
        self.idx += n;
        Ok(range)
    }

    /// Read `n` bytes as a String
    fn read_exact_as_string(
        &mut self,
        n: usize,
        string: &mut String,
    ) -> Result<(), TypedStreamError> {
        let str = std::str::from_utf8(self.read_exact_bytes(n)?)
            .map_err(TypedStreamError::StringParseError)?;
        string.push_str(str);
        Ok(())
    }

    /// Get the byte at a given index, if the index is within the bounds of the `typedstream`
    fn get_byte(&self, byte_idx: usize) -> Result<u8, TypedStreamError> {
        if byte_idx < self.stream.len() {
            return Ok(self.stream[byte_idx]);
        }
        Err(TypedStreamError::OutOfBounds(byte_idx, self.stream.len()))
    }

    /// Read the current byte
    fn get_current_byte(&self) -> Result<u8, TypedStreamError> {
        self.get_byte(self.idx)
    }

    /// Read the next byte
    fn get_next_byte(&self) -> Result<u8, TypedStreamError> {
        self.get_byte(self.idx + 1)
    }

    /// Read some bytes as an array
    fn read_array(&mut self, size: usize) -> Result<Vec<u8>, TypedStreamError> {
        Ok(self.read_exact_bytes(size)?.to_vec())
    }

    /// Determine the current types
    fn read_type(&mut self) -> Result<Vec<Type>, TypedStreamError> {
        let length = self.read_unsigned_int()?;

        let types = self.read_exact_bytes(length as usize)?;

        // Handle array size
        if types.first() == Some(&0x5b) {
            return Type::get_array_length(types).ok_or(TypedStreamError::InvalidArray);
        }

        Ok(types.iter().map(Type::from_byte).collect())
    }

    /// Read a reference pointer for a Type
    fn read_pointer(&mut self) -> Result<u32, TypedStreamError> {
        let pointer = self.get_current_byte()?;
        let result = u32::from(pointer)
            .checked_sub(REFERENCE_TAG as u32)
            .ok_or(TypedStreamError::InvalidPointer(pointer));
        self.idx += 1;
        result
    }

    /// Read a class
    fn read_class(&mut self) -> Result<ClassResult, TypedStreamError> {
        let mut out_v: Vec<Archivable> = vec![];
        match self.get_current_byte()? {
            START => {
                // Skip some header bytes
                while self.get_current_byte()? == START {
                    self.idx += 1;
                }
                let length = self.read_unsigned_int()?;

                if length >= REFERENCE_TAG {
                    let index = length - REFERENCE_TAG;
                    return Ok(ClassResult::Index(index as usize));
                }

                let mut class_name = String::with_capacity(length as usize);
                self.read_exact_as_string(length as usize, &mut class_name)?;

                let version = self.read_unsigned_int()?;

                self.types_table
                    .push(vec![Type::new_string(class_name.clone())]);

                out_v.push(Archivable::Class(Class::new(class_name, version)));

                if let ClassResult::ClassHierarchy(parent) = self.read_class()? {
                    out_v.extend(parent);
                }
            }
            EMPTY => {
                self.idx += 1;
            }
            _ => {
                let index = self.read_pointer()?;
                return Ok(ClassResult::Index(index as usize));
            }
        }
        Ok(ClassResult::ClassHierarchy(out_v))
    }

    /// Read an object into the cache and emit, or emit an already-cached object
    fn read_object(&mut self) -> Result<Option<&Archivable>, TypedStreamError> {
        match self.get_current_byte()? {
            START => {
                match self.read_class()? {
                    ClassResult::Index(idx) => {
                        return Ok(self.object_table.get(idx));
                    }
                    ClassResult::ClassHierarchy(classes) => {
                        for class in classes {
                            self.object_table.push(class);
                        }
                    }
                }
                Ok(None)
            }
            EMPTY => {
                self.idx += 1;
                Ok(None)
            }
            _ => {
                let index = self.read_pointer()?;
                Ok(self.object_table.get(index as usize))
            }
        }
    }

    /// Read String data
    fn read_string(&mut self) -> Result<String, TypedStreamError> {
        let length = self.read_unsigned_int()?;
        let mut string = String::with_capacity(length as usize);
        self.read_exact_as_string(length as usize, &mut string)?;

        Ok(string)
    }

    /// [`Archivable`] data can be embedded on a class or in a C String marked as [`Type::EmbeddedData`]
    fn read_embedded_data(&mut self) -> Result<Option<Archivable>, TypedStreamError> {
        // Skip the 0x84
        self.idx += 1;
        match self.get_type(true)? {
            Some(types) => self.read_types(types),
            None => Ok(None),
        }
    }

    /// Gets the current type from the stream, either by reading it from the stream or reading it from
    /// the specified index of [`TypedStreamReader::types_table`]. Because methods that use this type can also mutate self,
    /// returning a reference here means other methods could make that reference to the table invalid,
    /// which is disallowed in Rust. Thus, we return a clone of the cached data.
    fn get_type(&mut self, embedded: bool) -> Result<Option<Vec<Type>>, TypedStreamError> {
        match self.get_current_byte()? {
            START => {
                // Ignore repeated types, for example in a dict
                self.idx += 1;

                let object_types = self.read_type()?;

                // Embedded data is stored as a C String in the objects table
                if embedded {
                    self.object_table
                        .push(Archivable::Type(object_types.clone()));
                    // We only want to include the first embedded reference tag, not subsequent references to the same embed
                    self.seen_embedded_types
                        .insert(self.object_table.len().saturating_sub(1) as u32);
                }
                self.types_table.push(object_types);
                Ok(self.types_table.last().cloned())
            }
            END => {
                // This indicates the end of the current object
                Ok(None)
            }
            _ => {
                // Ignore repeated types, for example in a dict
                while self.get_current_byte()? == self.get_next_byte()? {
                    self.idx += 1;
                }

                let ref_tag = self.read_pointer()?;
                let result = self.types_table.get(ref_tag as usize);

                if embedded {
                    if let Some(res) = result {
                        // We only want to include the first embedded reference tag, not subsequent references to the same embed
                        if !self.seen_embedded_types.contains(&ref_tag) {
                            self.object_table.push(Archivable::Type(res.clone()));
                            self.seen_embedded_types.insert(ref_tag);
                        }
                    }
                }

                Ok(result.cloned())
            }
        }
    }

    /// Given some [`Type`]s, look at the stream and parse the data according to the specified [`Type`]
    fn read_types(
        &mut self,
        found_types: Vec<Type>,
    ) -> Result<Option<Archivable>, TypedStreamError> {
        let mut out_v = vec![];
        let mut is_obj: bool = false;

        for found_type in found_types {
            match found_type {
                Type::Utf8String => out_v.push(OutputData::String(self.read_string()?)),
                Type::EmbeddedData => {
                    return self.read_embedded_data();
                }
                Type::Object => {
                    is_obj = true;
                    let length = self.object_table.len();
                    self.placeholder = Some(length);
                    self.object_table.push(Archivable::Placeholder);
                    if let Some(object) = self.read_object()? {
                        match object.clone() {
                            Archivable::Object(_, data) => {
                                // If this is a new object, i.e. one without any data, we add the data into it later
                                // If the object already has data in it, we just want to return that object
                                if !data.is_empty() {
                                    let result = Ok(Some(object.clone()));
                                    self.placeholder = None;
                                    self.object_table.pop();
                                    return result;
                                }
                                out_v.extend(data);
                            }
                            Archivable::Class(cls) => out_v.push(OutputData::Class(cls)),
                            Archivable::Data(data) => out_v.extend(data),
                            // These cases are used internally in the objects table but should not be present in any output
                            Archivable::Placeholder | Archivable::Type(_) => {}
                        }
                    }
                }
                Type::SignedInt => out_v.push(OutputData::SignedInteger(self.read_signed_int()?)),
                Type::UnsignedInt => {
                    out_v.push(OutputData::UnsignedInteger(self.read_unsigned_int()?));
                }
                Type::Float => out_v.push(OutputData::Float(self.read_float()?)),
                Type::Double => out_v.push(OutputData::Double(self.read_double()?)),
                Type::Unknown(byte) => out_v.push(OutputData::Byte(byte)),
                Type::String(s) => out_v.push(OutputData::String(s)),
                Type::Array(size) => out_v.push(OutputData::Array(self.read_array(size)?)),
            }
        }

        // If we had reserved a place for an object, fill that spot
        if let Some(spot) = self.placeholder {
            if !out_v.is_empty() {
                // We got a class, but do not have its respective data yet
                if let Some(OutputData::Class(class)) = out_v.last() {
                    self.object_table[spot] = Archivable::Object(class.clone(), vec![]);
                // The spot after the current placeholder contains the class at the top of the class heirarchy, i.e.
                // if we get a placeholder and then find a new class heirarchy, the object table holds the class chain
                // in descending order of inheritance
                } else if let Some(Archivable::Class(class)) = self.object_table.get(spot + 1) {
                    self.object_table[spot] = Archivable::Object(class.clone(), out_v.clone());
                    self.placeholder = None;
                    return Ok(self.object_table.get(spot).cloned());
                // We got some data for a class that was already seen
                } else if let Some(Archivable::Object(_, data)) = self.object_table.get_mut(spot) {
                    data.extend(out_v.clone());
                    self.placeholder = None;
                    return Ok(self.object_table.get(spot).cloned());
                // We got some data that is not part of a class, i.e. a field in the parent object for which we don't know the name
                } else {
                    self.object_table[spot] = Archivable::Data(out_v.clone());
                    self.placeholder = None;
                    return Ok(self.object_table.get(spot).cloned());
                }
            }
        }

        if !out_v.is_empty() && !is_obj {
            return Ok(Some(Archivable::Data(out_v.clone())));
        }
        Ok(None)
    }

    /// In the original source there are several variants of the header, but we
    /// only need to validate that this is the header used by macOS/iOS, as iMessage
    /// is probably not available on any `NeXT` platform
    pub(crate) fn validate_header(&mut self) -> Result<(), TypedStreamError> {
        // Encoding type
        let typedstream_version = self.read_unsigned_int()?;
        // Encoding signature
        let signature = self.read_string()?;
        // System version
        let system_version = self.read_signed_int()?;

        if typedstream_version != 4 || signature != "streamtyped" || system_version != 1000 {
            return Err(TypedStreamError::InvalidHeader);
        }

        Ok(())
    }

    /// Attempt to get the data from the `typedstream`.
    ///
    /// Given a stream, construct a reader object to parse it. `typedstream` data doesn't include property
    /// names, so data is stored on [`Object`](crate::util::typedstream::models::Archivable::Object)s in order of appearance.
    ///
    /// Yields a new [`Archivable`] as they occur in the stream, but does not retain the object's inheritance heirarchy.
    /// Callers are responsible for assembling the deserialized stream into a useful data structure.
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::typedstream::parser::TypedStreamReader;
    ///
    /// let bytes: Vec<u8> = vec![]; // Example stream
    /// let mut reader = TypedStreamReader::from(&bytes);
    /// let result = reader.parse();
    /// ```
    ///
    /// # Sample output:
    /// ```txt
    /// [
    ///     Object(Class { name: "NSMutableString", version: 1 }, [String("Example")]) // The message text
    ///     Data([Integer(1), Integer(7)])  // The next object describes properties for the range of chars 1 through 7
    ///     Object(Class { name: "NSDictionary", version: 0 }, [Integer(1)])  // The first property is a `NSDictionary` with 1 item
    ///     Object(Class { name: "NSString", version: 1 }, [String("__kIMMessagePartAttributeName")])  // The first key in the `NSDictionary`
    ///     Object(Class { name: "NSNumber", version: 0 }, [Integer(0)])  // The first value in the `NSDictionary`
    /// ]
    /// ```
    pub fn parse(&mut self) -> Result<Vec<Archivable>, TypedStreamError> {
        let mut out_v = vec![];

        self.validate_header()?;

        while self.idx < self.stream.len() {
            if self.get_current_byte()? == END {
                self.idx += 1;
                continue;
            }

            // First, get the current type
            if let Some(found_types) = self.get_type(false)? {
                let result = self.read_types(found_types);
                if let Ok(Some(res)) = result {
                    out_v.push(res);
                }
            }
        }

        Ok(out_v)
    }
}
