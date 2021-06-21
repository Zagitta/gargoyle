use core::f64;

use super::super::codec::DecodedMessage;
use super::error::{Error, Result};
use bytes::Bytes;
use serde::de::{self, IntoDeserializer, Visitor};

use simdutf8::basic::from_utf8;

use std;
use std::convert::TryFrom;

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de DecodedMessage,
    index: usize,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_msg(input: &'de DecodedMessage) -> Self {
        Deserializer { input, index: 0 }
    }
    fn parse_f64(&mut self) -> Result<f64> {
        match self.peek_utf8_str() {
            Ok(s) => {
                if s.is_empty() {
                    Ok(0.0)
                } else {
                    s.parse::<f64>()
                        .map_err(|e| {
                            println!("Failed to parse f64");
                            serde::de::Error::custom(e)
                        })
                        .and_then(|r| {
                            self.advance();
                            Ok(r)
                        })
                }
            }
            Err(e) => Err(e),
        }
    }
    fn parse_f32(&mut self) -> Result<f32> {
        match self.peek_utf8_str() {
            Ok(s) => {
                if s.is_empty() {
                    Ok(0.0)
                } else {
                    s.parse::<f32>()
                        .map_err(|e| {
                            println!("Failed to parse f32");
                            serde::de::Error::custom(e)
                        })
                        .and_then(|r| {
                            self.advance();
                            Ok(r)
                        })
                }
            }
            Err(e) => Err(e),
        }
    }

    fn parse_i32(&mut self) -> Result<i32> {
        match self.peek_utf8_str() {
            Ok(s) => {
                if s.is_empty() {
                    Ok(0)
                } else {
                    s.parse::<i32>()
                        .map_err(|e| {
                            println!("Failed to parse i32");
                            serde::de::Error::custom(e)
                        })
                        .and_then(|r| {
                            self.advance();
                            Ok(r)
                        })
                }
            }
            Err(e) => Err(e),
        }
    }

    fn parse_i64(&mut self) -> Result<i64> {
        match self.peek_utf8_str() {
            Ok(s) => {
                if s.is_empty() {
                    Ok(0)
                } else {
                    s.parse::<i64>()
                        .map_err(|e| {
                            println!("Failed to parse i64");
                            serde::de::Error::custom(e)
                        })
                        .and_then(|r| {
                            self.advance();
                            Ok(r)
                        })
                }
            }
            Err(e) => Err(e),
        }
    }

    fn parse_bool(&mut self) -> Result<bool> {
        match self.parse_i32() {
            Ok(i) => {
                self.advance();
                Ok(i == 0)
            }
            Err(e) => Err(e),
        }
    }

    fn peek_utf8_str(&mut self) -> Result<&'de str> {
        match self.input.get(self.index) {
            Some(bytes) => from_utf8(&bytes[..]).map_err(|e| {
                println!("Failed to parse utf8");
                serde::de::Error::custom(format!("{:?}", e))
            }),
            None => Err(Error::Eof),
        }
    }

    fn get_utf8_str(&mut self) -> Result<&'de str> {
        match self.peek_utf8_str() {
            s @ Ok(_) => {
                self.advance();
                s
            }
            e @ Err(_) => e,
        }
    }

    fn current_is_empty(&self) -> Result<bool> {
        match self.input.get(self.index) {
            Some(bytes) => Ok(bytes.is_empty()),
            None => Ok(true),
        }
    }

    fn take_bytes(&mut self) -> Result<Bytes> {
        match self.input.get(self.index) {
            Some(bytes) => {
                let b = bytes.clone();
                self.advance();
                Ok(b)
            }
            None => Err(Error::Eof),
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }
}

impl<'de> Drop for Deserializer<'de> {
    fn drop(&mut self) {
        //debug_assert_eq!(self.index, self.input.len())
        if self.index + 1 < self.input.len() {
            println!(
                "Failed to fully deserialize >>{:?}<< only consumed {:?} elements",
                self.input, self.index
            );
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_i64()?)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_f32()?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_f64()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.get_utf8_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(&self.take_bytes()?[..])
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(&self.take_bytes()?[..])
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.current_is_empty()? {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.advance();
        visitor.visit_unit()
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let vec_size = usize::try_from(self.parse_i32()?).map_err(|_| Error::Syntax)?;

        if vec_size > (self.input.len() - self.index) {
            return Err(Error::Eof);
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len: vec_size,
        })
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        struct Access<'a, 'de> {
            deserializer: &'a mut Deserializer<'de>,
            len: usize,
        }

        impl<'a, 'de> serde::de::SeqAccess<'de> for Access<'a, 'de> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: serde::de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        impl<'a, 'de> serde::de::EnumAccess<'de> for &'a mut Deserializer<'de> {
            type Error = Error;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                let idx = self.get_utf8_str()?;
                let val: Result<_> = seed.deserialize(idx.into_deserializer());
                Ok((val?, self))
            }
        }

        visitor.visit_enum(self)
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Access<'a, 'de> {
    deserializer: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> serde::de::SeqAccess<'de> for Access<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'a, 'de> serde::de::VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        serde::de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Deserializer, Error};
    use bytes::Bytes;
    use bytestring::ByteString;
    use serde::Deserialize;
    #[test]
    fn can_deser_struct_lifetime() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test<'a> {
            int: i32,
            s: &'a str,
        }
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            Test {
                int: 1,
                s: "foobar"
            },
            Test::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_bytes() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: i32,
            s: Bytes,
        }
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            Test {
                int: 1,
                s: "foobar".into()
            },
            Test::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_bytestring() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: i32,
            s: ByteString,
        }
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            Test {
                int: 1,
                s: "foobar".into()
            },
            Test::deserialize(&mut de).unwrap()
        );
    }

    #[test]
    fn can_deser_nested_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test2 {
            s: String,
        }
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: i32,
            inner: Test2,
        }
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            Test {
                int: 1,
                inner: Test2 {
                    s: "foobar".to_owned()
                }
            },
            Test::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_tuple() {
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        type Tup = (i32, String);

        assert_eq!(
            (1i32, "foobar".to_owned()),
            Tup::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_enum() {
        let msg = vec!["1".into(), "1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        enum Enum {
            #[serde(rename = "1")]
            First { int: i32, s: String },
            #[serde(rename = "2")]
            Second { s: String, int: i32 },
        };

        assert_eq!(
            Enum::First {
                int: 1i32,
                s: "foobar".to_owned()
            },
            Enum::deserialize(&mut de).unwrap()
        );
    }

    #[test]
    fn can_deser_empty_enum() {
        let msg = vec!["3".into()];
        let mut de = Deserializer::from_msg(&msg);

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        enum Enum {
            #[serde(rename = "1")]
            First { int: i32, s: String },
            #[serde(rename = "2")]
            Second { s: String, int: i32 },
            #[serde(rename = "3")]
            Empty,
        };

        assert_eq!(Enum::Empty, Enum::deserialize(&mut de).unwrap());
    }

    #[test]
    fn can_deser_option_some() {
        let msg = vec!["1".into(), "foobar".into()];
        let mut de = Deserializer::from_msg(&msg);

        type Tup = (i32, Option<String>);

        assert_eq!(
            (1i32, Some("foobar".to_owned())),
            Tup::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_option_none() {
        let msg = vec!["1".into(), "".into()];
        let mut de = Deserializer::from_msg(&msg);

        type Tup = (i32, Option<String>);

        assert_eq!((1i32, None), Tup::deserialize(&mut de).unwrap());
    }
    #[test]
    fn can_deser_vec() {
        let msg = vec!["2".into(), "1".into(), "2".into()];
        let mut de = Deserializer::from_msg(&msg);

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Data {
            elms: Vec<String>,
        }

        assert_eq!(
            Data {
                elms: vec!["1".into(), "2".into()]
            },
            Data::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn can_deser_vec_of_tup() {
        let msg = vec!["1".into(), "1".into(), "2".into()];
        let mut de = Deserializer::from_msg(&msg);

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Data {
            elms: Vec<(String, String)>,
        }

        assert_eq!(
            Data {
                elms: vec![("1".into(), "2".into())]
            },
            Data::deserialize(&mut de).unwrap()
        );
    }
    #[test]
    fn cannot_deser_too_short_vec() {
        let msg = vec!["2".into(), "1".into()];
        let mut de = Deserializer::from_msg(&msg);

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Data {
            elms: Vec<String>,
        }

        assert_eq!(Err(Error::Eof), Data::deserialize(&mut de));
    }
    #[test]
    fn cannot_deser_missing_tup_field() {
        let msg = vec!["1".into()];
        let mut de = Deserializer::from_msg(&msg);

        type Tup = (i32, String);

        assert_eq!(Err(Error::Eof), Tup::deserialize(&mut de));
    }
}
