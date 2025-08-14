/*!
 Data structures and models used by the `typedstream` parser.
*/

/// Represents a class stored in the `typedstream`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    /// The name of the class
    pub name: String,
    /// The encoded version of the class
    pub version: u64,
}

impl Class {
    pub(crate) fn new(name: String, version: u64) -> Self {
        Self { name, version }
    }
}

/// Rust structures containing data stored in the `typedstream`
#[derive(Debug, Clone, PartialEq)]
pub enum OutputData {
    /// Text data, denoted in the stream by [`Type::String`]
    String(String),
    /// Signed integer types are coerced into this container, denoted in the stream by [`Type::SignedInt`]
    SignedInteger(i64),
    /// Unsigned integer types are coerced into this container, denoted in the stream by [`Type::UnsignedInt`]
    UnsignedInteger(u64),
    /// Floating point numbers, denoted in the stream by [`Type::Float`]
    Float(f32),
    /// Double precision floats, denoted in the stream by [`Type::Double`]
    Double(f64),
    /// Bytes whose type is not known, denoted in the stream by [`Type::Unknown`]
    Byte(u8),
    /// Arbitrary collection of bytes in an array, denoted in the stream by [`Type::Array`]
    Array(Vec<u8>),
    /// A found class, in order of inheritance, used by [`Archivable::Class`]
    Class(Class),
}

/// Types of data that can be archived into the `typedstream`
#[derive(Debug, Clone, PartialEq)]
pub enum Archivable {
    /// An instance of a class that may contain some embedded data. `typedstream` data doesn't include property
    /// names, so data is stored in order of appearance.
    Object(Class, Vec<OutputData>),
    /// Some data that is likely a property on the object described by the `typedstream` but not part of a class.
    Data(Vec<OutputData>),
    /// A class referenced in the `typedstream`, usually part of an inheritance heirarchy that does not contain any data itself.
    Class(Class),
    /// A placeholder, only used when reserving a spot in the objects table for a reference to be filled with read class information.
    /// In a `typedstream`, the classes are stored in order of inheritance, so the top-level class described by the `typedstream`
    /// comes before the ones it inherits from. To preserve the order, we reserve the first slot to store the actual object's data
    /// and then later add it back to the right place.
    Placeholder,
    /// A type that made it through the parsing process without getting replaced by an object.
    Type(Vec<Type>),
}

impl Archivable {
    /// If `self` is an [`Object`](Archivable::Object) that contains a [`Class`] named `NSString` or `NSMutableString`,
    /// extract a [`&str`](str) from the associated [`Data`](Archivable::Data).
    ///
    /// # Example
    ///
    /// ```
    /// use imessage_database::util::typedstream::models::{Archivable, Class, OutputData};
    ///
    /// let nsstring = Archivable::Object(
    ///     Class {
    ///         name: "NSString".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::String("Hello world".to_string())]
    /// );
    /// println!("{:?}", nsstring.as_nsstring()); // Some("Hello world")
    ///
    /// let not_nsstring = Archivable::Object(
    ///     Class {
    ///         name: "NSNumber".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::SignedInteger(100)]
    /// );
    /// println!("{:?}", not_nsstring.as_nsstring()); // None
    /// ```
    #[must_use]
    pub fn as_nsstring(&self) -> Option<&str> {
        if let Archivable::Object(Class { name, .. }, value) = self {
            if name == "NSString" || name == "NSMutableString" {
                if let Some(OutputData::String(text)) = value.first() {
                    return Some(text);
                }
            }
        }
        None
    }

    /// If `self` is an [`Object`](Archivable::Object) that contains a [`Class`] named `NSNumber` pointing to a [`SignedInteger`](OutputData::SignedInteger),
    /// extract an [`i64`] from the [`Data`](Archivable::Data).
    ///
    /// # Example
    ///
    /// ```
    /// use imessage_database::util::typedstream::models::{Archivable, Class, OutputData};
    ///
    /// let nsnumber = Archivable::Object(
    ///     Class {
    ///         name: "NSNumber".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::SignedInteger(100)]
    /// );
    /// println!("{:?}", nsnumber.as_nsnumber_int()); // Some(100)
    ///
    /// let not_nsnumber = Archivable::Object(
    ///     Class {
    ///         name: "NSString".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::String("Hello world".to_string())]
    /// );
    /// println!("{:?}", not_nsnumber.as_nsnumber_int()); // None
    /// ```
    #[must_use]
    pub fn as_nsnumber_int(&self) -> Option<&i64> {
        if let Archivable::Object(Class { name, .. }, value) = self {
            if name == "NSNumber" {
                if let Some(OutputData::SignedInteger(num)) = value.first() {
                    return Some(num);
                }
            }
        }
        None
    }

    /// If `self` is an [`Object`](Archivable::Object) that contains a [`Class`] named `NSNumber` pointing to a [`Double`](OutputData::Double),
    /// extract an [`f64`] from the [`Data`](Archivable::Data).
    ///
    /// # Example
    ///
    /// ```
    /// use imessage_database::util::typedstream::models::{Archivable, Class, OutputData};
    ///
    /// let nsnumber = Archivable::Object(
    ///     Class {
    ///         name: "NSNumber".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::Double(100.001)]
    /// );
    /// println!("{:?}", nsnumber.as_nsnumber_float()); // Some(100.001)
    ///
    /// let not_nsnumber = Archivable::Object(
    ///     Class {
    ///         name: "NSString".to_string(),
    ///         version: 1
    ///     },
    ///     vec![OutputData::String("Hello world".to_string())]
    /// );
    /// println!("{:?}", not_nsnumber.as_nsnumber_float()); // None
    /// ```
    #[must_use]
    pub fn as_nsnumber_float(&self) -> Option<&f64> {
        if let Archivable::Object(Class { name, .. }, value) = self {
            if name == "NSNumber" {
                if let Some(OutputData::Double(num)) = value.first() {
                    return Some(num);
                }
            }
        }
        None
    }
}

/// Represents primitive types of data that can be stored in a `typedstream`
///
/// These type encodings are partially documented [here](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html#//apple_ref/doc/uid/TP40008048-CH100-SW1) by Apple.
// TODO: Remove clone
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Encoded string data, usually embedded in an object. Denoted by:
    ///
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x28` | [`+`](https://www.compart.com/en/unicode/U+002B) |
    Utf8String,
    /// Encoded bytes that can be parsed again as data. Denoted by:
    ///
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x2A` | [`*`](https://www.compart.com/en/unicode/U+002A) |
    EmbeddedData,
    /// An instance of a class, usually with data. Denoted by:
    ///
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x40` | [`@`](https://www.compart.com/en/unicode/U+0040) |
    Object,
    /// An [`i8`], [`i16`], or [`i32`]. Denoted by:
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x63` | [`c`](https://www.compart.com/en/unicode/U+0063) |
    /// | `0x69` | [`i`](https://www.compart.com/en/unicode/U+0069) |
    /// | `0x6c` | [`l`](https://www.compart.com/en/unicode/U+006c) |
    /// | `0x71` | [`q`](https://www.compart.com/en/unicode/U+0071) |
    /// | `0x73` | [`s`](https://www.compart.com/en/unicode/U+0073) |
    ///
    /// The width is determined by the prefix: [`i8`] has none, [`i16`] has `0x81`, and [`i32`] has `0x82`.
    SignedInt,
    /// A [`u8`], [`u16`], or [`u32`]. Denoted by:
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x43` | [`C`](https://www.compart.com/en/unicode/U+0043) |
    /// | `0x49` | [`I`](https://www.compart.com/en/unicode/U+0049) |
    /// | `0x4c` | [`L`](https://www.compart.com/en/unicode/U+004c) |
    /// | `0x51` | [`Q`](https://www.compart.com/en/unicode/U+0051) |
    /// | `0x53` | [`S`](https://www.compart.com/en/unicode/U+0053) |
    ///
    /// The width is determined by the prefix: [`u8`] has none, [`u16`] has `0x81`, and [`u32`] has `0x82`.
    UnsignedInt,
    /// An [`f32`]. Denoted by:
    ///
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x66` | [`f`](https://www.compart.com/en/unicode/U+0066) |
    Float,
    /// An [`f64`]. Denoted by:
    ///
    /// | Hex    | UTF-8 |
    /// |--------|-------|
    /// | `0x64` | [`d`](https://www.compart.com/en/unicode/U+0064) |
    Double,
    /// Some text we can reuse later, i.e. a class name.
    String(String),
    /// An array containing some data of a given length. Denoted by braced digits: `[123]`.
    Array(usize),
    /// Data for which we do not know the type, likely for something this parser does not implement.
    Unknown(u8),
}

impl Type {
    pub(crate) fn from_byte(byte: &u8) -> Self {
        match byte {
            0x40 => Self::Object,
            0x2B => Self::Utf8String,
            0x2A => Self::EmbeddedData,
            0x66 => Self::Float,
            0x64 => Self::Double,
            0x63 | 0x69 | 0x6c | 0x71 | 0x73 => Self::SignedInt,
            0x43 | 0x49 | 0x4c | 0x51 | 0x53 => Self::UnsignedInt,
            other => Self::Unknown(*other),
        }
    }

    pub(crate) fn new_string(string: String) -> Self {
        Self::String(string)
    }

    pub(crate) fn get_array_length(types: &[u8]) -> Option<Vec<Type>> {
        if types.first() == Some(&0x5b) {
            let len =
                types[1..]
                    .iter()
                    .take_while(|a| a.is_ascii_digit())
                    .fold(None, |acc, ch| {
                        char::from_u32(u32::from(*ch))?
                            .to_digit(10)
                            .map(|b| acc.unwrap_or(0) * 10 + b)
                    })?;
            return Some(vec![Type::Array(len as usize)]);
        }
        None
    }
}

/// Represents data that results from attempting to parse a class from the `typedstream`
#[derive(Debug)]
pub(crate) enum ClassResult {
    /// A reference to an already-seen class in the [`TypedStreamReader::object_table`](crate::util::typedstream::parser::TypedStreamReader::object_table)
    Index(usize),
    /// A new class heirarchy to be inserted into the [`TypedStreamReader::object_table`](crate::util::typedstream::parser::TypedStreamReader::object_table)
    ClassHierarchy(Vec<Archivable>),
}
