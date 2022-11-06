#![allow(dead_code)]

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::error::ErrorKind;
use nom::number::Endianness;
use nom::number::complete::{u16, u32, u64};
use nom::sequence::{pair, tuple};
use strum_macros::{Display as DDisplay, FromRepr};
use tags::Tag;

mod tags;

#[derive(Debug, PartialEq)]
struct TifInfo {
    endianess: Endianness,
    big: bool,
}

fn big_endian (i: &[u8]) -> IResult<&[u8], Endianness> {
    let (input, _) = tag("MM")(i)?;
    Ok((input, Endianness::Big))
}

fn little_endian (i: &[u8]) -> IResult<&[u8], Endianness> {
    let (input, _) = tag("II")(i)?;
    Ok((input, Endianness::Little))
}

fn bigtiff(i: &[u8], endianess: Endianness) -> IResult<&[u8], bool> {
    let (input, magic_tiff_number) = u16(endianess)(i)?;
    match magic_tiff_number {
        42 => Ok((input, false)),
        43 => Ok((input, true)),
        _ => panic!("Invalid TIFF magic number!"),
    }
}

fn endianess(i: &[u8]) -> IResult<&[u8], Endianness> {
    alt((little_endian, big_endian))(i)
}

fn tif_header(i: &[u8]) -> IResult<&[u8], TifInfo> {
    let (input, endian) = endianess(i)?;
    let (input, is_big) = bigtiff(input, endian)?;
    Ok((input, TifInfo { endianess: endian, big: is_big }))
}

fn initial_parse(i: &[u8]) -> IResult<&[u8], (TifInfo, u64)> {
    let (remainder, info) = tif_header(i)?;
    // TODO: see if cond combinator can be used to check info.big
    if info.big {
        let two_bytes = |val| { u16(info.endianess)(val) };
        let (input, (offset_bytesize, zero_constant)) = pair(two_bytes, two_bytes)(remainder)?;
        assert_eq!(offset_bytesize, 8);
        assert_eq!(zero_constant, 0);
        let (input, offset) = u64(info.endianess)(input)?;
        Ok((input, (info, offset)))
    } else {
        let (input, offset) = u32(info.endianess)(remainder)?;
        Ok((input, (info, offset as u64)))
    }

}

fn first_ifd_count<'a>(i: &'a[u8], info: &TifInfo) -> IResult<&'a[u8], u64> {
    let result = if info.big {
        u64(info.endianess)(i)?
    } else {
        let (remainder, count) = u16(info.endianess)(i)?;
        (remainder, count as u64)
    };

    Ok(result)
}

#[derive(Debug, FromRepr, PartialEq)]
#[repr(u32)]
enum IfdEntryValue {
    Number(u32),
    Text([u8; 4]),
    Ascii(String),
    ArrayFloat,
    ArrayInteger,
}

#[derive(Debug, PartialEq)]
struct IfdEntry<'a> {
    tag_id: u64,
    type_: EntryType,
    count: u64,
    data: IfdEntryData<'a>,
    endian: &'a Endianness,
}

#[derive(Debug, PartialEq)]
enum IfdEntryData<'a> {
    Value(&'a[u8]),
    Reference(u32),
}

#[derive(Debug, DDisplay, FromRepr, PartialEq)]
#[repr(u16)]
enum EntryType {
    Byte = 1,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}

fn parse_ifd_id<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], u64> {
    let (remainder, tag_number) = u16(*endian)(i)?;
    Ok((remainder, tag_number as u64))
    // let tag: Option<Tag> = (tag_number as u64).try_into().ok();
    // Ok((remainder, tag))
}

fn parse_ifd_tag<'a>(tag_number: u64) -> Option<Tag> {
    tag_number.try_into().ok()
}

fn parse_entry_type<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], Option<EntryType>> {
    let (remainder, entry_type) = u16(*endian)(i)?;
    Ok((remainder, EntryType::from_repr(entry_type)))
}

impl<'a> Display for IfdEntry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let data_value = match self.data {
            IfdEntryData::Value(v) => v,
            IfdEntryData::Reference(_) => return Ok(write!(f, "Entry: {:?}", self))?,
        };

        let tag: Tag = match self.tag_id.try_into() {
            Ok(t) => t,
            Err(_) => panic!("Invalid tag! ({})", self.tag_id),
        };

        match self.type_ {
            EntryType::Byte | EntryType::Ascii => {
                match str::from_utf8(data_value) {
                    Ok(data) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, data))?,
                    Err(_) => panic!("Invalid string value!"),
                }
            },
            EntryType::Short => {
                match self.count {
                    1 => {
                        match u16::<_, (_, ErrorKind)>(*self.endian)(data_value) {
                            Ok((_, n)) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, n))?,
                            Err(_) => panic!("{:?}: Erroneous Short value!", tag),
                        }
                    },
                    2 => {
                        match tuple::<_, _, (_, ErrorKind), _>((u16(*self.endian), u16(*self.endian)))(data_value) {
                            Ok((_, (m, n))) => Ok(write!(f, "{:?} ({}): {}, {}", tag, self.tag_id, m, n))?,
                            Err(_) => panic!("{:?}: Erroneous Short value!", tag),
                        }
                    },
                    _ => panic!("{:?}: Too many entries of Short values!", tag)

                }
            },
            EntryType::Long => {
                match u32::<_, (_, ErrorKind)>(*self.endian)(data_value) {
                    Ok((_, n)) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, n))?,
                    Err(_) => panic!("{:?}: Erroneous Short value!", tag),
                }
            },
            EntryType::Rational => {
                Ok(write!(f, "{:?}: Rational", tag))?
                // let (remainder, (numerator, denomenator)) = tuple((u32(*self.endian), u32(*self.endian)))(i)?;
                // Ok(write!(f, "{:?}: {}/{}", tag, numerator, denomenator))?
            },
            // EntryType::Rational => 8,
            // EntryType::SByte => 1,
            // EntryType::Undefined => 1,
            // EntryType::SShort => 2,
            // EntryType::SLong => 4,
            // EntryType::SRational => 8,
            // EntryType::Float => 4,
            // EntryType::Double => 8,
            // t => Ok(write!(f, "{:?} ({:?}))?: unimplemented", tag, t)
            _ => Ok(write!(f, "{:?}: Rational", tag))?,
        }
    }
}

fn byte_width(entry_type: &EntryType) -> u8 {
    match entry_type {
        EntryType::Byte | EntryType::SByte | EntryType::Undefined | EntryType::Ascii => 1,
        EntryType::Short | EntryType::SShort => 2,
        EntryType::Long | EntryType::SLong | EntryType::Float => 4,
        EntryType::Rational | EntryType::SRational | EntryType::Double => 8,
    }
}

fn parse_entry_data<'a>(
    i: &'a[u8],
    endian: &Endianness,
    type_: &EntryType,
    count: u32,
) -> IResult<&'a[u8], IfdEntryData<'a>> {
    if (byte_width(type_) as u32 * count) > 4 {
        let (remainder, position) = u32(*endian)(i)?;
        Ok((remainder, IfdEntryData::Reference(position)))
    } else {
        let (remainder, bytes) = take(4u8)(i)?;
        Ok((remainder, IfdEntryData::Value(bytes)))
    }
}

fn parse_ifd_entry<'a>(i: &'a[u8], endian: &'a Endianness) -> IResult<&'a[u8], Option<IfdEntry<'a>>> {
    let (input, tag_id) = parse_ifd_id(i, endian)?;
    let (input, entry_type) = parse_entry_type(input, endian)?;
    let (input, count) = u32(*endian)(input)?;
    let tag: Option<Tag> = tag_id.try_into().ok();
    match (tag, entry_type) {
        (Some(_tag), Some(type_)) => {
            let (input, data) = parse_entry_data(input, endian, &type_, count)?;
            let result = IfdEntry{
                tag_id,
                type_,
                count: count as u64,
                data,
                endian,
            };
            Ok((input, Some(result)))
        },
        _ => Ok((input, None)),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("B09.tif");

    let mut file = File::open(&path).expect("error opening file");
    let mut buf = [0u8; 16];
    file.read_exact(&mut buf)?;

    match initial_parse(&buf) {
        Ok((_, (info, offset))) => {
            dbg!(&info);
            dbg!(offset);
            file.seek(SeekFrom::Start(offset))?;
            let mut counter_buf = [0u8; 8];

            if info.big {
                file.read_exact(&mut counter_buf)?;
            } else {
                let mut handle = file.by_ref().take(2);
                handle.read(&mut counter_buf)?;
            }

            if let Ok((_, count)) = first_ifd_count(&counter_buf, &info) {
                dbg!(count);

                if info.endianess == Endianness::Big {
                    let mut ifd_buf = [0u8; 20];
                    for _ in 0..count {
                        file.read_exact(&mut ifd_buf)?;
                    }
                } else {
                    for _ in 0..count {
                        let mut ifd_buf = [0u8; 12];
                        file.read_exact(&mut ifd_buf)?;
                        // dbg!(&ifd_buf);
                        if let Ok((_, Some(entry))) = parse_ifd_entry(&ifd_buf, &info.endianess) {
                            println!("{}", entry);
                        }
                    }
                };
            }
        },

        Err(_) => return Err(<&str as Into<Box<dyn std::error::Error>>>::into("Failed to parse TIF file!")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endianess() {
        assert_eq!(endianess(&b"MMabcefg"[..]), Ok((&b"abcefg"[..], Endianness::Big)));
        assert_eq!(endianess(&b"IIabcefg"[..]), Ok((&b"abcefg"[..], Endianness::Little)));
    }

    #[test]
    fn test_bigtiff() {
        assert_eq!(bigtiff(&b"\x00\x2Babcefg"[..], Endianness::Big), Ok((&b"abcefg"[..], true)));
        assert_eq!(bigtiff(&b"\x2B\x00abcefg"[..], Endianness::Little), Ok((&b"abcefg"[..], true)));
        assert_eq!(bigtiff(&b"\x00\x2Aabcefg"[..], Endianness::Big), Ok((&b"abcefg"[..], false)));
        assert_eq!(bigtiff(&b"\x2A\x00abcefg"[..], Endianness::Little), Ok((&b"abcefg"[..], false)));
    }

    #[test]
    fn test_parse_tiff_header_big_endian() {
        assert_eq!(tif_header(&b"MM\x00\x2Aabcefg"[..]), Ok((&b"abcefg"[..], TifInfo{endianess: Endianness::Big, big: false})));
    }

    #[test]
    fn test_parse_bigtiff_header_little_endian() {
        assert_eq!(tif_header(&b"II\x2B\x00abcefg"[..]), Ok((&b"abcefg"[..], TifInfo{endianess: Endianness::Little, big: true})));
    }

    #[test]
    fn test_parse_bigtiff_ifd_position() {
        let info = TifInfo{endianess: Endianness::Little, big: true};
        assert_eq!(
            initial_parse(&b"II\x2B\x00\x08\x00\x00\x00\xEF\xBE\x00\x00\x00\x00\x00\x00abcefg"[..]),
            Ok((&b"abcefg"[..], (info, 0xbeef)))
        );
    }

    #[test]
    fn test_parse_tiff_ifd_position() {
        let info = TifInfo{endianess: Endianness::Little, big: false};
        assert_eq!(
            initial_parse(&b"II\x2A\x00\xEF\xBE\x00\x00abcefg"[..]),
            Ok((&b"abcefg"[..], (info, 0xbeef)))
        );
    }

    #[test]
    fn test_parse_ifd_count() {
        let info = TifInfo{endianess: Endianness::Little, big: false};
        assert_eq!(
            first_ifd_count(&b"\xEF\xBEabcefg"[..], &info),
            Ok((&b"abcefg"[..], 0xbeef))
        );
    }

    #[test]
    fn test_parse_ifd_entry() {
        assert_eq!(
            parse_ifd_entry(
                &b"\x00\x01\x01\x00\x01\x00\x00\x00\x00\x00\x01\x00"[..],
                &Endianness::Little,
            ),
            Ok((
                &b""[..],
                Some(IfdEntry{
                    tag_id: Tag::ImageWidth as u64,
                    type_: EntryType::Byte,
                    count: 1,
                    data: IfdEntryData::Value(&[0, 0, 1, 0]),
                    endian: &Endianness::Little,
                }),
            ))
        );
    }

    #[test]
    fn test_parse_ifd_type() {
        assert_eq!(
            parse_entry_type(&b"\x01\x00"[..], &Endianness::Little),
            Ok((&b""[..], Some(EntryType::Byte)))
        );
        assert_eq!(
            parse_entry_type(&b"\x0c\x00"[..], &Endianness::Little),
            Ok((&b""[..], Some(EntryType::Double)))
        );
    }

    #[test]
    fn test_parse_bad_ifd_type_zero() {
        assert_eq!(
            parse_entry_type(&b"\x00\x00"[..], &Endianness::Little),
            Ok((&b""[..], None))
        );
    }

    #[test]
    fn test_parse_bad_ifd_type_excessive() {
        assert_eq!(
            parse_entry_type(&b"\x0d\x00"[..], &Endianness::Little),
            Ok((&b""[..], None))
        );
    }

    #[test]
    fn test_parse_bad_type_ifd_entry() {
        assert_eq!(
            parse_ifd_entry(
                &b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"[..],
                &Endianness::Little,
            ),
            Ok((&b"\x00\x00\x00\x00"[..], None))
        );
    }

    // #[test]
    // fn test_parse_ifd_data() {
    //     assert_eq!(
    //         parse_ifd_data(&b"\x01\x00\x00\x00"[..], &Endianness::Little),
    //         Err(nom::Err::Error(nom::error::Error::new(&b""[..], nom::error::ErrorKind::Fail)))
    //     );
    // }
}
