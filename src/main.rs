#![allow(dead_code)]

// use std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
// use nom::error::ErrorKind;
use nom::number::Endianness;
use nom::number::complete::{u16, u32, u64};
use nom::sequence::pair;
// use nom::sequence::{pair, tuple};
use strum_macros::{Display as DDisplay, FromRepr};

mod tags;
use tags::{Tag, TagInfoValue};

const IFD_BUF_SIZE: usize = 12;
const BIG_IFD_BUF_SIZE: usize = 20;

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

#[derive(Debug, PartialEq)]
struct TagInfo<T> {
    name: tags::Tag,
    value: TagInfoValue<T>
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

// impl<'a, T> TryFrom<IfdEntry<'a>> for TagInfo<T> {
//     type Error = ();

//     fn try_from(entry: IfdEntry) -> Result<Self, Self::Error> {
//         let data = match entry.data {
//             IfdEntryData::Value(v) => v,
//             IfdEntryData::Reference(_) => return Err(()),
//         };
//         Ok(match entry.tag_id {
//             // 254 => {
//             //     TagInfo{
//             //         name: tags::Tag::NewSubfileType,
//             //         value: TagInfoValue::Bitfield(data),
//             //         // value: TagInfoValue::<&[u8]>::Bitfield(data),
//             //     }
//             // },
//             // 255 => {
//             //     TagInfo{
//             //         name: tags::Tag::SubfileType,
//             //         value: TagInfoValue::Bitfield(data.into()),
//             //         // value: TagInfoValue::<&[u8]>::Bitfield(data),
//             //     }
//             // },
//             _ => unimplemented!(""),
//             // 255 => {SubfileType},
//             // 256 => {ImageWidth},
//             // 257 => {ImageLength},
//             // 258 => {BitsPerSample},
//             // 259 => {Compression},
//             // 262 => {PhotometricInterpretation},
//             // 263 => {Threshholding},
//             // 264 => {CellWidth},
//             // 265 => {CellLength},
//             // 266 => {FillOrder},
//             // 269 => {DocumentName},
//             // 270 => {ImageDescription},
//             // 271 => {Make},
//             // 272 => {Model},
//             // 273 => {StripOffsets},
//             // 274 => {Orientation},
//             // 277 => {SamplesPerPixel},
//             // 278 => {RowsPerStrip},
//             // 279 => {StripByteCounts},
//             // 280 => {MinSampleValue},
//             // 281 => {MaxSampleValue},
//             // 282 => {XResolution},
//             // 283 => {YResolution},
//             // 284 => {PlanarConfiguration},
//             // 285 => {PageName},
//             // 286 => {XPosition},
//             // 287 => {YPosition},
//             // 288 => {FreeOffsets},
//             // 289 => {FreeByteCounts},
//             // 290 => {GrayResponseUnit},
//             // 291 => {GrayResponseCurve},
//             // 292 => {OptionsT4},
//             // 293 => {OptionsT6},
//             // 296 => {ResolutionUnit},
//             // 297 => {PageNumber},
//             // 301 => {TransferFunction},
//             // 305 => {Software},
//             // 306 => {DateTime},
//             // 315 => {Artist},
//             // 316 => {HostComputer},
//             // 317 => {Predictor},
//             // 318 => {WhitePoint},
//             // 319 => {PrimaryChromaticities},
//             // 320 => {ColorMap},
//             // 321 => {HalftoneHints},
//             // 322 => {TileWidth},
//             // 323 => {TileLength},
//             // 324 => {TileOffsets},
//             // 325 => {TileByteCounts},
//             // 326 => {BadFaxLines},
//             // 327 => {CleanFaxData},
//             // 328 => {ConsecutiveBadFaxLines},
//             // 330 => {SubIFDs},
//             // 332 => {InkSet},
//             // 333 => {InkNames},
//             // 334 => {NumberOfInks},
//             // 336 => {DotRange},
//             // 337 => {TargetPrinter},
//             // 338 => {ExtraSamples},
//             // 339 => {SampleFormat},
//             // 340 => {SMinSampleValue},
//             // 341 => {SMaxSampleValue},
//             // 342 => {TransferRange},
//             // 343 => {ClipPath},
//             // 344 => {XClipPathUnits},
//             // 345 => {YClipPathUnits},
//             // 346 => {Indexed},
//             // 347 => {JPEGTables},
//             // 351 => {OPIProxy},
//             // 400 => {GlobalParametersIFD},
//             // 401 => {ProfileType},
//             // 402 => {FaxProfile},
//             // 403 => {CodingMethods},
//             // 404 => {VersionYear},
//             // 405 => {ModeNumber},
//             // 433 => {Decode},
//             // 434 => {DefaultImageColor},
//             // 512 => {JPEGProc},
//             // 513 => {JPEGInterchangeFormat},
//             // 514 => {JPEGInterchangeFormatLength},
//             // 515 => {JPEGRestartInterval},
//             // 517 => {JPEGLosslessPredictors},
//             // 518 => {JPEGPointTransforms},
//             // 519 => {JPEGQTables},
//             // 520 => {JPEGDCTables},
//             // 521 => {JPEGACTables},
//             // 529 => {YCbCrCoefficients},
//             // 530 => {YCbCrSubSampling},
//             // 531 => {YCbCrPositioning},
//             // 532 => {ReferenceBlackWhite},
//             // 559 => {StripRowCounts},
//             // 700 => {XMP},
//             // 32781 => {ImageID},
//             // 33432 => {Copyright},
//             // 33550 => {ModelPixelScale},
//             // 33922 => {Georeference},
//             // 34377 => {Photoshop},
//             // 34732 => {ImageLayer},
//             // 34735 => {GeoKeyDirectory},
//             // 34737 => {GeoAsciiParams},
//             // 42112 => {GdalMetadata},
//             // 42113 => {GdalNoData},

//         })
//     }
// }

// impl<'a> Display for IfdEntry<'a> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//         let data_value = match self.data {
//             IfdEntryData::Value(v) => v,
//             IfdEntryData::Reference(_) => return Ok(write!(f, "Entry: {:?}", self))?,
//         };

//         let tag: Tag = match self.tag_id.try_into() {
//             Ok(t) => t,
//             Err(_) => panic!("Invalid tag! ({})", self.tag_id),
//         };

//         match self.type_ {
//             EntryType::Byte | EntryType::Ascii => {
//                 match str::from_utf8(data_value) {
//                     Ok(data) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, data))?,
//                     Err(_) => panic!("Invalid string value!"),
//                 }
//             },
//             EntryType::Short => {
//                 match self.count {
//                     1 => {
//                         match u16::<_, (_, ErrorKind)>(*self.endian)(data_value) {
//                             Ok((_, n)) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, n))?,
//                             Err(_) => panic!("{:?}: Erroneous Short value!", tag),
//                         }
//                     },
//                     2 => {
//                         match tuple::<_, _, (_, ErrorKind), _>((u16(*self.endian), u16(*self.endian)))(data_value) {
//                             Ok((_, (m, n))) => Ok(write!(f, "{:?} ({}): {}, {}", tag, self.tag_id, m, n))?,
//                             Err(_) => panic!("{:?}: Erroneous Short value!", tag),
//                         }
//                     },
//                     _ => panic!("{:?}: Too many entries of Short values!", tag)

//                 }
//             },
//             EntryType::Long => {
//                 match u32::<_, (_, ErrorKind)>(*self.endian)(data_value) {
//                     Ok((_, n)) => Ok(write!(f, "{:?} ({}): {}", tag, self.tag_id, n))?,
//                     Err(_) => panic!("{:?}: Erroneous Short value!", tag),
//                 }
//             },
//             EntryType::Rational => {
//                 Ok(write!(f, "{:?}: Rational", tag))?
//                 // let (remainder, (numerator, denomenator)) = tuple((u32(*self.endian), u32(*self.endian)))(i)?;
//                 // Ok(write!(f, "{:?}: {}/{}", tag, numerator, denomenator))?
//             },
//             // EntryType::Rational => 8,
//             // EntryType::SByte => 1,
//             // EntryType::Undefined => 1,
//             // EntryType::SShort => 2,
//             // EntryType::SLong => 4,
//             // EntryType::SRational => 8,
//             // EntryType::Float => 4,
//             // EntryType::Double => 8,
//             // t => Ok(write!(f, "{:?} ({:?}))?: unimplemented", tag, t)
//             _ => Ok(write!(f, "{:?}: Rational", tag))?,
//         }
//     }
// }

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
    tag: tags::Tag,
    endian: &Endianness,
    type_: &EntryType,
    count: u32,
) -> IResult<&'a[u8], IfdEntryData<'a>> {
    if (byte_width(type_) as u32 * count) > 4 {
        let (remainder, position) = u32(*endian)(i)?;
        Ok((remainder, IfdEntryData::Reference(position)))
    } else {
        let (remainder, bytes) = take(4u8)(i)?;
        match tag {
            _ => Ok((remainder, IfdEntryData::Value(bytes)))
        }
    }
}

fn parse_ifd_entry<'a>(i: &'a[u8], endian: &'a Endianness) -> IResult<&'a[u8], Option<IfdEntry<'a>>> {
    let (input, tag_id) = parse_ifd_id(i, endian)?;
    let (input, entry_type) = parse_entry_type(input, endian)?;
    let (input, count) = u32(*endian)(input)?;
    let maybe_tag: Option<Tag> = tag_id.try_into().ok();
    match (maybe_tag, entry_type) {
        (Some(tag), Some(type_)) => {
            let (input, data) = parse_entry_data(input, tag, endian, &type_, count)?;
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
                    let mut ifd_buf = [0u8; BIG_IFD_BUF_SIZE];
                    for _ in 0..count {
                        file.read_exact(&mut ifd_buf)?;
                    }
                } else {
                    let mut ifd_buf = [0u8; IFD_BUF_SIZE];
                    let mut ifds = BTreeMap::new();
                    let mut ifd_refs = Vec::new();
                    for _ in 0..count {
                        file.read_exact(&mut ifd_buf)?;
                        // dbg!(&ifd_buf);
                        let ifd_buf_copy = ifd_buf.clone();
                        if let Ok((_, Some(entry))) = parse_ifd_entry(&ifd_buf_copy, &info.endianess) {
                            match entry.data {
                                IfdEntryData::Value(buffer) => {
                                    // let tag = entry
                                    ifds.insert(
                                        entry.tag_id,
                                        TagInfo {
                                            name: entry.tag_id.try_into()?,
                                            value:  TagInfoValue::Number(32),
                                        },
                                    );
                                },
                                IfdEntryData::Reference(_position) => {
                                    ifd_refs.push(ifd_buf_copy);
                                },
                            };
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
                    // data: IfdEntryData::Value(tags::TagValue::ImageWidth(256)),
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
