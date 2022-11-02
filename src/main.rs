#![allow(dead_code)]

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::number::Endianness;
use nom::number::complete::{u16, u32, u64, u8};
use nom::sequence::{pair, tuple};
use strum_macros::{Display, FromRepr};
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

// fn read_ifd_count<'a>(file_offset: u64, file: &File, info: &'a TifInfo) -> Result<u64, Box<dyn std::error::Error>> {
//

// fn read_ifd_count<'a, 'b>(file_offset: u64, file: &mut File, info: &'a TifInfo) -> IResult<&'a[u8], u64> {
//     file.seek(SeekFrom::Start(file_offset)).expect("Reading IFD count from file failed!");
//     let mut buf = [0u8; 8];
// {

//     let (_, count) = if info.big {
//         file.read_exact(&mut buf).expect("Reading IFD count from file failed!");
//         alt((u64(info.endianess), fail::<_, u64, _>))(&buf[..])?
//     } else {
//         let mut handle = file.take(2);
//         handle.read(&mut buf).expect("Reading IFD count from file failed!");
//         let (remainder, count) = alt((u16(info.endianess), fail::<_, u16, _>))(&buf[..])?;
//         (remainder, count as u64)
//     };
//     Ok((&b""[..], count.clone()))
//     }
// }

fn first_ifd_count<'a>(i: &'a[u8], info: &TifInfo) -> IResult<&'a[u8], u64> {
    let result = if info.big {
        u64(info.endianess)(i)?
    } else {
        let (remainder, count) = u16(info.endianess)(i)?;
        (remainder, count as u64)
    };

    Ok(result)
}

#[derive(Debug, PartialEq)]
enum IfdEntryData {
    Value((u8, u8, u8, u8)),
    Reference(u32),
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
struct IfdEntry {
    tag: Tag,
    type_: EntryType,
    count: u64,
    data: IfdEntryData,
}

#[derive(Debug, Display, FromRepr, PartialEq)]
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

fn parse_ifd_tag<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], Option<Tag>> {
    let (remainder, tag_number) = u16(*endian)(i)?;
    let tag: Option<Tag> = tag_number.try_into().ok();
    Ok((remainder, tag))
}

fn parse_entry_type<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], Option<EntryType>> {
    let (remainder, entry_type) = u16(*endian)(i)?;
    Ok((remainder, EntryType::from_repr(entry_type)))
}

// fn position_to_string<'a>(
//     i: &'a [u8],
//     endian: &Endianness,
//     tag: &Tag,
//     type_: &EntryType,
//     count: u32,
// ) -> IResult<&'a [u8], String> {
//     unimplemented!();
// }

fn entry_to_string<'a>(entry: &IfdEntry) -> Option<String> {
    let data_value = match entry.data {
        IfdEntryData::Value(v) => v,
        IfdEntryData::Reference(_) => return None,
    };
    match entry.type_ {
        EntryType::Byte | EntryType::Ascii => {
            let data = match str::from_utf8(data_value) {
                Ok(v) => v,
                Err => panic!("Invalid string value!"),
            };
            format!("{:?}: {:?}", entry.tag, data)
        },
        EntryType::Short => {
            "".to_owned()
            // let (remainder, data) = many_m_n(1, count as usize, u16(*endian))(i)?;
            // format!("{:?}: {:?}", tag, data)
        },
        EntryType::Long => {
            "".to_owned()
            // let (remainder, data) = many_m_n(1, count as usize, u32(*endian))(i)?;
            // format!("{:?}: {:?}", tag, data)
        },
        EntryType::Rational => {
            "".to_owned()
            // let (remainder, (numerator, denomenator)) = tuple((u32(*endian), u32(*endian)))(i)?;
            // format!("{:?}: {}/{}", tag, numerator, denomenator)
        },
        // EntryType::Rational => 8,
        // EntryType::SByte => 1,
        // EntryType::Undefined => 1,
        // EntryType::SShort => 2,
        // EntryType::SLong => 4,
        // EntryType::SRational => 8,
        // EntryType::Float => 4,
        // EntryType::Double => 8,
        // t => format!("{:?} ({:?}): unimplemented", tag, t)
        _ => "".to_owned()
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
) -> IResult<&'a[u8], IfdEntryData> {
    if (byte_width(type_) as u32 * count) > 4 {
        let (remainder, position) = u32(*endian)(i)?;
        Ok((remainder, IfdEntryData::Reference(position)))
    } else {
        let (remainder, bytes) = tuple((u8, u8, u8, u8))(i)?;
        Ok((remainder, IfdEntryData::Value(bytes)))
    }
}

// fn display_ifd_entry<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], Option<String>> {
//     let (input, tag) = parse_ifd_tag(i, endian)?;
//     let (input, entry_type) = parse_entry_type(input, endian)?;
//     let (input, count) = u32(*endian)(input)?;
//     match (tag, entry_type) {
//         (Some(tag), Some(type_)) => {
//             let (remainder, data) = parse_entry_data(input, endian, &tag, &type_, count)?;

//             // if (byte_width(&type_) as u32 * count) > 4 {
//             //     let (input, data) = position_to_string(input, endian, &tag, &type_, count)?;
//             //     return Ok((i, IfdEntryData::Position(0)))
//             // }
//             // let (input, data) = entry_to_string(input, endian, &tag, &type_, count)?;
//             // Ok((input, Some(data)))
//         },
//         _ => Ok((input, None)),
//     }
// }

fn parse_ifd_entry<'a>(i: &'a[u8], endian: &Endianness) -> IResult<&'a[u8], Option<IfdEntry>> {
    let (input, tag) = parse_ifd_tag(i, endian)?;
    let (input, entry_type) = parse_entry_type(input, endian)?;
    let (input, count) = u32(*endian)(input)?;
    match (tag, entry_type) {
        (Some(tag), Some(type_)) => {
            let (input, data) = parse_entry_data(input, endian, &type_, count)?;
            let result = IfdEntry{
                tag,
                type_,
                count: count as u64,
                data,
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

                if info.big {
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
                            dbg!(entry);
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
                    tag: Tag::ImageWidth,
                    type_: EntryType::Byte,
                    count: 1,
                    data: IfdEntryData::Value((0, 0, 1, 0)),
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
