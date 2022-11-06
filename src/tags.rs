use strum_macros::{Display, FromRepr};

macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<u64> for $name {
            type Error = String;

            fn try_from(v: u64) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u64 => Ok($name::$vname),)*
                    // TODO: conform to TIFF spec by skipping the tag! But warn the user!
                    _ => Err(format!("Invalid tag: {} !", v)),
                }
            }
        }
    }
}

back_to_enum! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Tag {
        NewSubfileType = 254,	 // A general indication of the kind of data contained in this subfile.
        SubfileType = 255,	 // A general indication of the kind of data contained in this subfile.
        ImageWidth = 256,	 // The number of columns in the image, i.e., the number of pixels per row.
        ImageLength = 257,	 // The number of rows of pixels in the image.
        BitsPerSample = 258,	 // Number of bits per component.
        Compression = 259,	 // Compression scheme used on the image data.
        PhotometricInterpretation = 262,	 // The color space of the image data.
        Threshholding = 263,  // 	For black and white TIFF files that represent shades of gray, the technique used to convert from gray to black and white pixels.
        CellWidth = 264,  // 	The width of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
        CellLength = 265,  // 	The length of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
        FillOrder = 266,	 // The logical order of bits within a byte.
        DocumentName = 269,	// The name of the document from which this image was scanned.
        ImageDescription = 270,	 // A string that describes the subject of the image.
        Make = 271,	 // The scanner manufacturer.
        Model = 272,	 // The scanner model name or number.
        StripOffsets = 273,	 // For each strip, the byte offset of that strip.
        Orientation = 274,	 // The orientation of the image with respect to the rows and columns.
        SamplesPerPixel = 277,	 // The number of components per pixel.
        RowsPerStrip = 278,	 // The number of rows per strip.
        StripByteCounts = 279,	 // For each strip, the number of bytes in the strip after compression.
        MinSampleValue = 280,	 // The minimum component value used.
        MaxSampleValue = 281,	 // The maximum component value used.
        XResolution = 282,	 // The number of pixels per ResolutionUnit in the ImageWidth direction.
        YResolution = 283,	 // The number of pixels per ResolutionUnit in the ImageLength direction.
        PlanarConfiguration = 284,	 // How the components of each pixel are stored.
        PageName = 285,	// The name of the page from which this image was scanned.
        XPosition = 286,	// X position of the image.
        YPosition = 287,	// Y position of the image.
        FreeOffsets = 288,	 // For each string of contiguous unused bytes in a TIFF file, the byte offset of the string.
        FreeByteCounts = 289,  // 	For each string of contiguous unused bytes in a TIFF file, the number of bytes in the string.
        GrayResponseUnit = 290,	 // The precision of the information contained in the GrayResponseCurve.
        GrayResponseCurve = 291,	 // For grayscale data, the optical density of each possible pixel value.
        OptionsT4 = 292,	// Options for Group 3 Fax compression
        OptionsT6 = 293,	// Options for Group 4 Fax compression
        ResolutionUnit = 296,	 // The unit of measurement for XResolution and YResolution.
        PageNumber = 297,	// The page number of the page from which this image was scanned.
        TransferFunction = 301,	// Describes a transfer function for the image in tabular style.
        Software = 305,	 // Name and version number of the software package(s) used to create the image.
        DateTime = 306,	 // Date and time of image creation.
        Artist = 315,	 // Person who created the image.
        HostComputer = 316,	 // The computer and/or operating system in use at the time of image creation.
        Predictor  = 317, 	// A mathematical operator that is applied to the image data before an encoding scheme is applied.
        WhitePoint = 318,	// The chromaticity of the white point of the image.
        PrimaryChromaticities = 319,	// The chromaticities of the primaries of the image.
        ColorMap = 320,	 // A color map for palette color images.
        HalftoneHints  = 321, 	// Conveys to the halftone function the range of gray levels within a colorimetrically-specified image that should retain tonal detail.
        TileWidth = 322,	// The tile width in pixels. This is the number of columns in each tile.
        TileLength = 323,	// The tile length (height) in pixels. This is the number of rows in each tile.
        TileOffsets = 324,	// For each tile, the byte offset of that tile, as compressed and stored on disk.
        TileByteCounts = 325,	// For each tile, the number of (compressed) bytes in that tile.
        BadFaxLines  = 326, 	// Used in the TIFF-F standard, denotes the number of 'bad' scan lines encountered by the facsimile device.
        CleanFaxData  = 327, 	// Used in the TIFF-F standard, indicates if 'bad' lines encountered during reception are stored in the data, or if 'bad' lines have been replaced by the receiver.
        ConsecutiveBadFaxLines  = 328, 	// Used in the TIFF-F standard, denotes the maximum number of consecutive 'bad' scanlines received.
        SubIFDs = 330,	// Offset to child IFDs.
        InkSet = 332,	// The set of inks used in a separated (PhotometricInterpretation=5) image.
        InkNames = 333,	// The name of each ink used in a separated image.
        NumberOfInks = 334,	// The number of inks.
        DotRange = 336,	// The component values that correspond to a 0% dot and 100% dot.
        TargetPrinter = 337,	// A description of the printing environment for which this separation is intended.
        ExtraSamples = 338,	 // Description of extra components.
        SampleFormat = 339,	// Specifies how to interpret each data sample in a pixel.
        SMinSampleValue = 340,	// Specifies the minimum sample value.
        SMaxSampleValue = 341,	// Specifies the maximum sample value.
        TransferRange = 342,	// Expands the range of the TransferFunction.
        ClipPath = 343,	// Mirrors the essentials of PostScript's path creation functionality.
        XClipPathUnits  = 344, 	// The number of units that span the width of the image, in terms of integer ClipPath coordinates.
        YClipPathUnits  = 345, 	// The number of units that span the height of the image, in terms of integer ClipPath coordinates.
        Indexed = 346,	// Aims to broaden the support for indexed images to include support for any color space.
        JPEGTables = 347,	// JPEG quantization and/or Huffman tables.
        OPIProxy = 351,	// OPI-related.
        GlobalParametersIFD  = 400, 	// Used in the TIFF-FX standard to point to an IFD containing tags that are globally applicable to the complete TIFF file.
        ProfileType = 401,	// Used in the TIFF-FX standard, denotes the type of data stored in this file or IFD.
        FaxProfile = 402,	// Used in the TIFF-FX standard, denotes the 'profile' that applies to this file.
        CodingMethods = 403,	// Used in the TIFF-FX standard, indicates which coding methods are used in the file.
        VersionYear  = 404, 	// Used in the TIFF-FX standard, denotes the year of the standard specified by the FaxProfile field.
        ModeNumber  = 405, 	// Used in the TIFF-FX standard, denotes the mode of the standard specified by the FaxProfile field.
        Decode  = 433, 	// Used in the TIFF-F and TIFF-FX standards, holds information about the ITULAB (PhotometricInterpretation = 10) encoding.
        DefaultImageColor  = 434, 	// Defined in the Mixed Raster Content part of RFC 2301, is the default color needed in areas where no image is available.
        JPEGProc = 512,	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGInterchangeFormat  = 513, 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGInterchangeFormatLength  = 514, 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGRestartInterval  = 515, 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGLosslessPredictors  = 517, 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGPointTransforms  = 518, 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGQTables = 519,	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGDCTables = 520,	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        JPEGACTables = 521,	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
        YCbCrCoefficients = 529,	// The transformation from RGB to YCbCr image data.
        YCbCrSubSampling  = 530, 	// Specifies the subsampling factors used for the chrominance components of a YCbCr image.
        YCbCrPositioning  = 531, 	// Specifies the positioning of subsampled chrominance components relative to luminance samples.
        ReferenceBlackWhite  = 532, 	// Specifies a pair of headroom and footroom image data values (codes) for each pixel component.
        StripRowCounts  = 559, 	// Defined in the Mixed Raster Content part of RFC 2301, used to replace RowsPerStrip for IFDs with variable-sized strips.
        XMP = 700,	// XML packet containing XMP metadata
        ImageID = 32781,	// OPI-related.
        Copyright = 33432,	 // Copyright notice.
        ModelPixelScale = 33550,  //  Exact affine transformations between raster and model space.
        Georeference = 33922,  //  raster->model tiepoint pairs
        Photoshop = 34377,  // Collection of Photoshop 'Image Resource Blocks'.
        ImageLayer  = 34732, 	// Defined in the Mixed Raster Content part of RFC 2301, used to denote the particular function of this Image in the mixed raster scheme.
        GeoKeyDirectory = 34735, // GeoKey Directory, which defines and references the "GeoKeys"
        GeoAsciiParams = 34737,  // ASCII valued GeoKeys, referenced by the GeoKeyDirectoryTag
        GdalMetadata = 42112,  // xml maps keys Item.name to Item tag text as value
        GdalNoData = 42113,  // ASCII represented pixel value interpreted as transparent
    }
}

pub enum TagValue {
    NewSubfileType(u32),	 // A general indication of the kind of data contained in this subfile.
    SubfileType(SubfileType),	 // A general indication of the kind of data contained in this subfile.
    ImageWidth(u32),	 // The number of columns in the image, i.e., the number of pixels per row.
    ImageLength(u32),	 // The number of rows of pixels in the image.
    BitsPerSample(u32),	 // Number of bits per component.
    Compression(Compression),	 // Compression scheme used on the image data.
    PhotometricInterpretation(PhotometricInterpretation),	 // The color space of the image data.
    Threshholding(u32),  // 	For black and white TIFF files that represent shades of gray, the technique used to convert from gray to black and white pixels.
    CellWidth(u32),  // 	The width of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
    CellLength(u32),  // 	The length of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
    FillOrder(u32),	 // The logical order of bits within a byte.
    DocumentName(u32),	// The name of the document from which this image was scanned.
    ImageDescription(u32),	 // A string that describes the subject of the image.
    Make(u32),	 // The scanner manufacturer.
    Model(u32),	 // The scanner model name or number.
    StripOffsets(u32),	 // For each strip, the byte offset of that strip.
    Orientation(u32),	 // The orientation of the image with respect to the rows and columns.
    SamplesPerPixel(u32),	 // The number of components per pixel.
    RowsPerStrip(u32),	 // The number of rows per strip.
    StripByteCounts(u32),	 // For each strip, the number of bytes in the strip after compression.
    MinSampleValue(u32),	 // The minimum component value used.
    MaxSampleValue(u32),	 // The maximum component value used.
    XResolution(u32),	 // The number of pixels per ResolutionUnit in the ImageWidth direction.
    YResolution(u32),	 // The number of pixels per ResolutionUnit in the ImageLength direction.
    PlanarConfiguration(PlanarConfiguration),	 // How the components of each pixel are stored.
    PageName(u32),	// The name of the page from which this image was scanned.
    XPosition(u32),	// X position of the image.
    YPosition(u32),	// Y position of the image.
    FreeOffsets(u32),	 // For each string of contiguous unused bytes in a TIFF file, the byte offset of the string.
    FreeByteCounts(u32),  // 	For each string of contiguous unused bytes in a TIFF file, the number of bytes in the string.
    GrayResponseUnit(u32),	 // The precision of the information contained in the GrayResponseCurve.
    GrayResponseCurve(u32),	 // For grayscale data, the optical density of each possible pixel value.
    OptionsT4(u32),	// Options for Group 3 Fax compression
    OptionsT6(u32),	// Options for Group 4 Fax compression
    ResolutionUnit(ResolutionUnit),	 // The unit of measurement for XResolution and YResolution.
    PageNumber(u32),	// The page number of the page from which this image was scanned.
    TransferFunction(u32),	// Describes a transfer function for the image in tabular style.
    Software(u32),	 // Name and version number of the software package(s) used to create the image.
    DateTime(u32),	 // Date and time of image creation.
    Artist(u32),	 // Person who created the image.
    HostComputer(u32),	 // The computer and/or operating system in use at the time of image creation.
    Predictor(Predictor), 	// A mathematical operator that is applied to the image data before an encoding scheme is applied.
    WhitePoint(u32),	// The chromaticity of the white point of the image.
    PrimaryChromaticities(u32),	// The chromaticities of the primaries of the image.
    ColorMap(u32),	 // A color map for palette color images.
    HalftoneHints(u32), 	// Conveys to the halftone function the range of gray levels within a colorimetrically-specified image that should retain tonal detail.
    TileWidth(u32),	// The tile width in pixels. This is the number of columns in each tile.
    TileLength(u32),	// The tile length(height) in pixels. This is the number of rows in each tile.
    TileOffsets(u32),	// For each tile, the byte offset of that tile, as compressed and stored on disk.
    TileByteCounts(u32),	// For each tile, the number of(compressed) bytes in that tile.
    BadFaxLines(u32), 	// Used in the TIFF-F standard, denotes the number of 'bad' scan lines encountered by the facsimile device.
    CleanFaxData(u32), 	// Used in the TIFF-F standard, indicates if 'bad' lines encountered during reception are stored in the data, or if 'bad' lines have been replaced by the receiver.
    ConsecutiveBadFaxLines(u32), 	// Used in the TIFF-F standard, denotes the maximum number of consecutive 'bad' scanlines received.
    SubIFDs(u32),	// Offset to child IFDs.
    InkSet(u32),	// The set of inks used in a separated(PhotometricInterpretation=5) image.
    InkNames(u32),	// The name of each ink used in a separated image.
    NumberOfInks(u32),	// The number of inks.
    DotRange(u32),	// The component values that correspond to a 0% dot and 100% dot.
    TargetPrinter(u32),	// A description of the printing environment for which this separation is intended.
    ExtraSamples(u32),	 // Description of extra components.
    SampleFormat(SampleFormat),	// Specifies how to interpret each data sample in a pixel.
    SMinSampleValue(u32),	// Specifies the minimum sample value.
    SMaxSampleValue(u32),	// Specifies the maximum sample value.
    TransferRange(u32),	// Expands the range of the TransferFunction.
    ClipPath(u32),	// Mirrors the essentials of PostScript's path creation functionality.
    XClipPathUnits(u32), 	// The number of units that span the width of the image, in terms of integer ClipPath coordinates.
    YClipPathUnits(u32), 	// The number of units that span the height of the image, in terms of integer ClipPath coordinates.
    Indexed(u32),	// Aims to broaden the support for indexed images to include support for any color space.
    JPEGTables(u32),	// JPEG quantization and/or Huffman tables.
    OPIProxy(u32),	// OPI-related.
    GlobalParametersIFD(u32), 	// Used in the TIFF-FX standard to point to an IFD containing tags that are globally applicable to the complete TIFF file.
    ProfileType(u32),	// Used in the TIFF-FX standard, denotes the type of data stored in this file or IFD.
    FaxProfile(u32),	// Used in the TIFF-FX standard, denotes the 'profile' that applies to this file.
    CodingMethods(u32),	// Used in the TIFF-FX standard, indicates which coding methods are used in the file.
    VersionYear(u32), 	// Used in the TIFF-FX standard, denotes the year of the standard specified by the FaxProfile field.
    ModeNumber(u32), 	// Used in the TIFF-FX standard, denotes the mode of the standard specified by the FaxProfile field.
    Decode(u32), 	// Used in the TIFF-F and TIFF-FX standards, holds information about the ITULAB (PhotometricInterpretation = 10) encoding.
    DefaultImageColor(u32), 	// Defined in the Mixed Raster Content part of RFC 2301, is the default color needed in areas where no image is available.
    JPEGProc(u32),	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGInterchangeFormat(u32), 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGInterchangeFormatLength(u32), 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGRestartInterval(u32), 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGLosslessPredictors(u32), 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGPointTransforms(u32), 	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGQTables(u32),	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGDCTables(u32),	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    JPEGACTables(u32),	// Old-style JPEG compression field. TechNote2 invalidates this part of the specification.
    YCbCrCoefficients(u32),	// The transformation from RGB to YCbCr image data.
    YCbCrSubSampling(u32), 	// Specifies the subsampling factors used for the chrominance components of a YCbCr image.
    YCbCrPositioning(u32), 	// Specifies the positioning of subsampled chrominance components relative to luminance samples.
    ReferenceBlackWhite(u32), 	// Specifies a pair of headroom and footroom image data values (codes) for each pixel component.
    StripRowCounts(u32), 	// Defined in the Mixed Raster Content part of RFC 2301, used to replace RowsPerStrip for IFDs with variable-sized strips.
    XMP(u32),	// XML packet containing XMP metadata
    ImageID(u32),	// OPI-related.
    Copyright(u32),	 // Copyright notice.
    ModelPixelScale(u32),  //  Exact affine transformations between raster and model space.
    Georeference(u32),  //  raster->model tiepoint pairs
    Photoshop(u32),  // Collection of Photoshop 'Image Resource Blocks'.
    ImageLayer(u32), 	// Defined in the Mixed Raster Content part of RFC 2301, used to denote the particular function of this Image in the mixed raster scheme.
    GeoKeyDirectory(u32), // GeoKey Directory, which defines and references the "GeoKeys"
    GeoAsciiParams(u32),  // ASCII valued GeoKeys, referenced by the GeoKeyDirectoryTag
    GdalMetadata(u32),  // xml maps keys Item.name to Item tag text as value
    GdalNoData(u32),  // ASCII represented pixel value interpreted as transparent
}

// trait TagResolve {
//     fn new() -> Self;
//     fn resolve(&self) -> TagValue;
// }

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum Compression {
    Uncompressed = 1,
    Ccitt1d,
    Group3Fax,
    Group4Fax,
    LZW,
    JpegOld,
    JpegNew,
    Deflate,
    PackBits = 32773,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
enum GeoKey {
    Standard = 0,
    ReducedResolution = 1,
    MultiPage = 2,
    TransparencyMask = 4,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum PhotometricInterpretation {
    WhiteIsZero,
    BlackIsZero,
    PaletteColor,
    RGB,
    TransparencyMask,
    YCbCr,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum PlanarConfiguration {
    Chunky = 1,
    Planar,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum Predictor {
    None = 1,
    Horizontal,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum ResolutionUnit {
    NoAbsoluteUnit = 1,
    Inch,
    Centimeter,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum SampleFormat {
    UnsignedInteger = 1,
    Signed2Compliment,
    FloatIEEE,
    Undefined,
}

#[derive(Display, FromRepr)]
#[repr(u32)]
pub enum SubfileType {
    Standard = 0,
    ReducedResolution = 1,
    MultiPage = 2,
    TransparencyMask = 4,
}

#[derive(Display, FromRepr)]
#[repr(u64)]
enum ModelTypeCode {
   Projected   = 1,   /* Projection Coordinate System         */
   Geographic  = 2,   /* Geographic latitude-longitude System */
   Geocentric  = 3,   /* Geocentric (X,Y,Z) Coordinate System */
}
