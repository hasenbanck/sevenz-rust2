use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

const K_END: u8 = 0x00;
const K_HEADER: u8 = 0x01;
const K_ARCHIVE_PROPERTIES: u8 = 0x02;
const K_ADDITIONAL_STREAMS_INFO: u8 = 0x03;
const K_MAIN_STREAMS_INFO: u8 = 0x04;
const K_FILES_INFO: u8 = 0x05;
const K_PACK_INFO: u8 = 0x06;
const K_UNPACK_INFO: u8 = 0x07;
const K_SUBSTREAMS_INFO: u8 = 0x08;
const K_SIZE: u8 = 0x09;
const K_CRC: u8 = 0x0A;
const K_FOLDER: u8 = 0x0B;
const K_CODERS_UNPACK_SIZE: u8 = 0x0C;
const K_NUM_UNPACK_STREAM: u8 = 0x0D;
const K_EMPTY_STREAM: u8 = 0x0E;
const K_EMPTY_FILE: u8 = 0x0F;
const K_ANTI: u8 = 0x10;
const K_NAME: u8 = 0x11;
const K_CTIME: u8 = 0x12;
const K_ATIME: u8 = 0x13;
const K_MTIME: u8 = 0x14;
const K_WIN_ATTRIBUTES: u8 = 0x15;
const K_ENCODED_HEADER: u8 = 0x17;
const K_START_POS: u8 = 0x18;
const K_DUMMY: u8 = 0x19;

fn property_id_to_string(id: u8) -> &'static str {
    match id {
        K_END => "kEnd",
        K_HEADER => "kHeader",
        K_ARCHIVE_PROPERTIES => "kArchiveProperties",
        K_ADDITIONAL_STREAMS_INFO => "kAdditionalStreamsInfo",
        K_MAIN_STREAMS_INFO => "kMainStreamsInfo",
        K_FILES_INFO => "kFilesInfo",
        K_PACK_INFO => "kPackInfo",
        K_UNPACK_INFO => "kUnPackInfo",
        K_SUBSTREAMS_INFO => "kSubStreamsInfo",
        K_SIZE => "kSize",
        K_CRC => "kCRC",
        K_FOLDER => "kFolder",
        K_CODERS_UNPACK_SIZE => "kCodersUnPackSize",
        K_NUM_UNPACK_STREAM => "kNumUnPackStream",
        K_EMPTY_STREAM => "kEmptyStream",
        K_EMPTY_FILE => "kEmptyFile",
        K_ANTI => "kAnti",
        K_NAME => "kName",
        K_CTIME => "kCTime",
        K_ATIME => "kATime",
        K_MTIME => "kMTime",
        K_WIN_ATTRIBUTES => "kWinAttributes",
        K_ENCODED_HEADER => "kEncodedHeader",
        K_START_POS => "kStartPos",
        K_DUMMY => "kDummy",
        _ => "Unknown",
    }
}

// Structure to hold stream info for passing between functions
struct StreamsInfo {
    num_folders: u64,
    folder_out_streams: Vec<u64>,
    folder_has_crc: Vec<bool>, // Track which folders have CRCs defined
}

struct SevenZipReader {
    file: File,
    indent: usize,
}

impl SevenZipReader {
    fn new(path: &str) -> Self {
        SevenZipReader {
            file: File::open(path).unwrap(),
            indent: 0,
        }
    }

    fn print_indent(&self) {
        for _ in 0..self.indent {
            print!("  ");
        }
    }

    fn println_info(&self, info: &str) {
        self.print_indent();
        println!("{}", info);
    }

    fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; size];
        self.file.read_exact(&mut buffer).unwrap();
        buffer
    }

    fn read_u8(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.file.read_exact(&mut buffer).unwrap();
        buffer[0]
    }

    fn read_u32(&mut self) -> u32 {
        let mut buffer = [0u8; 4];
        self.file.read_exact(&mut buffer).unwrap();
        u32::from_le_bytes(buffer)
    }

    fn read_u64(&mut self) -> u64 {
        let mut buffer = [0u8; 8];
        self.file.read_exact(&mut buffer).unwrap();
        u64::from_le_bytes(buffer)
    }

    fn read_encoded_uint64(&mut self) -> u64 {
        let first = self.read_u8() as u64;
        let mut mask = 0x80_u64;
        let mut value = 0;
        for i in 0..8 {
            if (first & mask) == 0 {
                return value | ((first & (mask - 1)) << (8 * i));
            }
            let b = self.read_u8() as u64;
            value |= b << (8 * i);
            mask >>= 1;
        }
        value
    }

    fn parse_signature_header(&mut self) {
        self.println_info("=== SIGNATURE HEADER ===");
        self.indent += 1;

        // Read signature.
        let signature = self.read_bytes(6);
        let expected = vec![b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

        self.print_indent();
        print!("Signature: ");
        for byte in &signature {
            print!("{:02X} ", byte);
        }

        if signature == expected {
            println!(" [VALID]");
        } else {
            println!(" [INVALID]");
            panic!("Invalid 7z signature!");
        }

        // Read version.
        let major = self.read_u8();
        let minor = self.read_u8();
        self.println_info(&format!("Version: {}.{}", major, minor));

        // Read Start Header CRC.
        let start_header_crc = self.read_u32();
        self.println_info(&format!("StartHeaderCRC: 0x{:08X}", start_header_crc));

        // Read Start Header.
        self.println_info("StartHeader:");
        self.indent += 1;

        let next_header_offset = self.read_u64();
        let next_header_size = self.read_u64();
        let next_header_crc = self.read_u32();

        self.println_info(&format!(
            "NextHeaderOffset: {} (0x{:X})",
            next_header_offset, next_header_offset
        ));
        self.println_info(&format!("NextHeaderSize: {} bytes", next_header_size));
        self.println_info(&format!("NextHeaderCRC: 0x{:08X}", next_header_crc));

        self.indent -= 2;

        let current_pos = 32;
        self.file
            .seek(SeekFrom::Start(current_pos + next_header_offset))
            .unwrap();

        self.parse_header(next_header_size as usize);
    }

    fn parse_header(&mut self, size: usize) {
        println!();
        self.println_info("=== MAIN HEADER ===");
        self.indent += 1;

        let start_pos = self.file.stream_position().unwrap();
        let end_pos = start_pos + size as u64;

        while self.file.stream_position().unwrap() < end_pos {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            self.println_info(&format!(
                "Property: {} (0x{:02X})",
                property_id_to_string(property_type),
                property_type
            ));

            self.indent += 1;

            match property_type {
                K_HEADER => {
                    self.parse_header_contents(end_pos);
                }
                K_ENCODED_HEADER => {
                    self.println_info("Encoded header detected");
                    self.parse_streams_info(end_pos);
                }
                K_ARCHIVE_PROPERTIES => {
                    self.parse_archive_properties();
                }
                K_ADDITIONAL_STREAMS_INFO => {
                    self.parse_streams_info(end_pos);
                }
                K_MAIN_STREAMS_INFO => {
                    self.parse_streams_info(end_pos);
                }
                K_FILES_INFO => {
                    self.parse_files_info();
                }
                _ => {
                    self.println_info(&format!("Unknown property type: 0x{:02X}", property_type));
                }
            }

            self.indent -= 1;
        }

        self.indent -= 1;
    }

    fn parse_header_contents(&mut self, end_pos: u64) {
        while self.file.stream_position().unwrap() < end_pos {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            self.println_info(&format!(
                "Sub-Property: {} (0x{:02X})",
                property_id_to_string(property_type),
                property_type
            ));

            self.indent += 1;

            match property_type {
                K_ARCHIVE_PROPERTIES => {
                    self.parse_archive_properties();
                }
                K_ADDITIONAL_STREAMS_INFO => {
                    self.parse_streams_info(end_pos);
                }
                K_MAIN_STREAMS_INFO => {
                    self.parse_streams_info(end_pos);
                }
                K_FILES_INFO => {
                    self.parse_files_info();
                }
                _ => {
                    self.println_info(&format!("Unknown property type: 0x{:02X}", property_type));
                }
            }

            self.indent -= 1;
        }
    }

    fn parse_archive_properties(&mut self) {
        loop {
            let property_type = self.read_u8();
            if property_type == 0 {
                self.println_info("End of archive properties");
                break;
            }

            let property_size = self.read_encoded_uint64();
            self.println_info(&format!(
                "Property Type: 0x{:02X}, Size: {} bytes",
                property_type, property_size
            ));

            self.file
                .seek(SeekFrom::Current(property_size as i64))
                .unwrap();
        }
    }

    fn parse_streams_info(&mut self, end_pos: u64) {
        let mut streams_info: Option<StreamsInfo> = None;

        while self.file.stream_position().unwrap() < end_pos {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            self.println_info(&format!(
                "StreamsInfo Property: {} (0x{:02X})",
                property_id_to_string(property_type),
                property_type
            ));

            self.indent += 1;

            match property_type {
                K_PACK_INFO => {
                    self.parse_pack_info();
                }
                K_UNPACK_INFO => {
                    streams_info = Some(self.parse_coders_info());
                }
                K_SUBSTREAMS_INFO => {
                    self.parse_substreams_info(streams_info.as_ref());
                }
                _ => {
                    self.println_info(&format!(
                        "Unknown streams info property: 0x{:02X}",
                        property_type
                    ));
                }
            }

            self.indent -= 1;
        }
    }

    fn parse_pack_info(&mut self) {
        let pack_pos = self.read_encoded_uint64();
        let num_pack_streams = self.read_encoded_uint64();

        self.println_info(&format!("PackPos: {}", pack_pos));
        self.println_info(&format!("NumPackStreams: {}", num_pack_streams));

        loop {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            match property_type {
                K_SIZE => {
                    self.println_info("Pack Sizes:");
                    self.indent += 1;
                    for i in 0..num_pack_streams {
                        let size = self.read_encoded_uint64();
                        self.println_info(&format!("Stream {}: {} bytes", i, size));
                    }
                    self.indent -= 1;
                }
                K_CRC => {
                    self.println_info("Pack CRCs:");
                    self.indent += 1;

                    let all_defined = self.read_u8();
                    let mut defined_flags = vec![true; num_pack_streams as usize];

                    if all_defined == 0 {
                        self.println_info("Not all CRCs defined, reading bit vector");
                        defined_flags.clear();

                        // Read bit vector.
                        let bytes_needed = num_pack_streams.div_ceil(8) as usize;
                        let bit_vector = self.read_bytes(bytes_needed);

                        for i in 0..num_pack_streams {
                            let byte_index = (i / 8) as usize;
                            let bit_index = (i % 8) as u8;
                            let is_defined = (bit_vector[byte_index] & (1 << bit_index)) != 0;
                            defined_flags.push(is_defined);
                        }
                    } else {
                        self.println_info("All CRCs defined");
                    }

                    // Read CRC values for defined streams.
                    for i in 0..num_pack_streams {
                        if defined_flags[i as usize] {
                            let crc = self.read_u32();
                            self.println_info(&format!("Stream {} CRC: 0x{:08X}", i, crc));
                        } else {
                            self.println_info(&format!("Stream {} CRC: undefined", i));
                        }
                    }

                    self.indent -= 1;
                }
                _ => {
                    self.println_info(&format!(
                        "Unknown pack info property: 0x{:02X}",
                        property_type
                    ));
                }
            }
        }
    }

    fn parse_coders_info(&mut self) -> StreamsInfo {
        let property_type = self.read_u8();

        if property_type != K_FOLDER {
            self.println_info(&format!("Expected kFolder, got: 0x{:02X}", property_type));
            return StreamsInfo {
                num_folders: 0,
                folder_out_streams: Vec::new(),
                folder_has_crc: Vec::new(),
            };
        }

        let num_folders = self.read_encoded_uint64();
        self.println_info(&format!("NumFolders: {}", num_folders));

        let external = self.read_u8();
        let mut folder_out_streams = Vec::new();

        if external == 0 {
            self.println_info("Folders data is internal");
            self.indent += 1;

            for folder_idx in 0..num_folders {
                self.println_info(&format!("Folder {}:", folder_idx));
                self.indent += 1;
                let out_streams = self.parse_folder();
                folder_out_streams.push(out_streams);
                self.indent -= 1;
            }

            self.indent -= 1;
        } else {
            let data_stream_index = self.read_encoded_uint64();
            self.println_info(&format!(
                "Folders data is external (DataStreamIndex: {})",
                data_stream_index
            ));
        }

        let mut folder_has_crc = vec![false; num_folders as usize];

        loop {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            match property_type {
                K_CODERS_UNPACK_SIZE => {
                    self.println_info("Coders Unpack Sizes:");
                    self.indent += 1;

                    for folder_idx in 0..num_folders {
                        let num_out_streams = if folder_out_streams.len() > folder_idx as usize {
                            folder_out_streams[folder_idx as usize]
                        } else {
                            1 // Default if we don't have the info.
                        };

                        self.println_info(&format!("Folder {}:", folder_idx));
                        self.indent += 1;

                        for stream_idx in 0..num_out_streams {
                            let size = self.read_encoded_uint64();
                            self.println_info(&format!("OutStream {}: {} bytes", stream_idx, size));
                        }

                        self.indent -= 1;
                    }

                    self.indent -= 1;
                }
                K_CRC => {
                    self.println_info("Folder CRCs:");
                    self.indent += 1;

                    let all_defined = self.read_u8();
                    let mut defined_flags = vec![true; num_folders as usize];

                    if all_defined == 0 {
                        self.println_info("Not all CRCs defined, reading bit vector");
                        defined_flags.clear();

                        // Read bit vector.
                        let bytes_needed = num_folders.div_ceil(8) as usize;
                        let bit_vector = self.read_bytes(bytes_needed);

                        for i in 0..num_folders {
                            let byte_index = (i / 8) as usize;
                            let bit_index = (i % 8) as u8;
                            let is_defined = (bit_vector[byte_index] & (1 << bit_index)) != 0;
                            defined_flags.push(is_defined);
                        }
                    } else {
                        self.println_info("All CRCs defined");
                    }

                    // Read CRC values for defined folders and mark them.
                    for i in 0..num_folders {
                        if defined_flags[i as usize] {
                            folder_has_crc[i as usize] = true;
                            let crc = self.read_u32();
                            self.println_info(&format!("Folder {} CRC: 0x{:08X}", i, crc));
                        } else {
                            self.println_info(&format!("Folder {} CRC: undefined", i));
                        }
                    }

                    self.indent -= 1;
                }
                _ => {
                    self.println_info(&format!(
                        "Unknown coders info property: 0x{:02X}",
                        property_type
                    ));
                }
            }
        }

        StreamsInfo {
            num_folders,
            folder_out_streams,
            folder_has_crc,
        }
    }

    fn parse_folder(&mut self) -> u64 {
        let num_coders = self.read_encoded_uint64();
        self.println_info(&format!("NumCoders: {}", num_coders));

        let mut total_out_streams = 0u64;
        let mut total_in_streams = 0u64;

        for coder_idx in 0..num_coders {
            self.println_info(&format!("Coder {}:", coder_idx));
            self.indent += 1;

            let flags = self.read_u8();
            let codec_id_size = flags & 0x0F;
            let is_complex = (flags & 0x10) != 0;
            let has_attributes = (flags & 0x20) != 0;

            self.println_info(&format!(
                "Flags: 0x{:02X} (Complex: {}, HasAttributes: {})",
                flags, is_complex, has_attributes
            ));

            let codec_id = self.read_bytes(codec_id_size as usize);
            self.print_indent();
            print!("CodecID: ");
            for byte in &codec_id {
                print!("{:02X} ", byte);
            }
            println!();

            let (in_streams, out_streams) = if is_complex {
                let num_in = self.read_encoded_uint64();
                let num_out = self.read_encoded_uint64();
                self.println_info(&format!(
                    "NumInStreams: {}, NumOutStreams: {}",
                    num_in, num_out
                ));
                (num_in, num_out)
            } else {
                (1, 1)
            };

            total_in_streams += in_streams;
            total_out_streams += out_streams;

            if has_attributes {
                let properties_size = self.read_encoded_uint64();
                self.println_info(&format!("PropertiesSize: {} bytes", properties_size));
                // Skip properties
                self.file
                    .seek(SeekFrom::Current(properties_size as i64))
                    .unwrap();
            }

            self.indent -= 1;
        }

        let num_bind_pairs = total_out_streams - 1;
        if num_bind_pairs > 0 {
            self.println_info(&format!("NumBindPairs: {}", num_bind_pairs));
            self.indent += 1;

            for i in 0..num_bind_pairs {
                let in_index = self.read_encoded_uint64();
                let out_index = self.read_encoded_uint64();
                self.println_info(&format!(
                    "BindPair {}: InIndex={}, OutIndex={}",
                    i, in_index, out_index
                ));
            }

            self.indent -= 1;
        }

        let num_packed_streams = total_in_streams - num_bind_pairs;
        if num_packed_streams > 1 {
            self.println_info(&format!(
                "NumPackedStreams: {} (indices follow)",
                num_packed_streams
            ));
            self.indent += 1;

            for i in 0..num_packed_streams {
                let index = self.read_encoded_uint64();
                self.println_info(&format!("PackedStream {}: Index={}", i, index));
            }

            self.indent -= 1;
        }

        total_out_streams
    }

    fn parse_substreams_info(&mut self, streams_info: Option<&StreamsInfo>) {
        // Get the number of folders from the streams_info if available
        let num_folders = streams_info.map(|si| si.num_folders).unwrap_or(0);

        if num_folders == 0 {
            self.println_info("WARNING: No folder information available for SubStreamsInfo");
            return;
        }

        // Track number of substreams per folder
        // Default to 1 substream per folder if not specified
        let mut num_unpack_streams_in_folders = vec![1u64; num_folders as usize];
        let mut total_substreams = num_folders; // Default assumption

        loop {
            let property_type = self.read_u8();

            if property_type == K_END {
                self.println_info("kEnd");
                break;
            }

            match property_type {
                K_NUM_UNPACK_STREAM => {
                    self.println_info("NumUnPackStream data:");
                    self.indent += 1;

                    // Clear defaults and read actual values
                    num_unpack_streams_in_folders.clear();
                    total_substreams = 0;

                    for folder_idx in 0..num_folders {
                        let num_streams = self.read_encoded_uint64();
                        self.println_info(&format!(
                            "Folder {}: {} substreams",
                            folder_idx, num_streams
                        ));

                        total_substreams += num_streams;
                        num_unpack_streams_in_folders.push(num_streams);
                    }

                    self.println_info(&format!(
                        "Total substreams across all folders: {}",
                        total_substreams
                    ));
                    self.indent -= 1;
                }
                K_SIZE => {
                    self.println_info("UnPack Sizes:");
                    self.indent += 1;

                    // For each folder with > 1 substream, read N-1 sizes
                    // The last size is calculated from folder's total unpack size
                    let mut size_idx = 0;
                    for (folder_idx, &num_streams) in
                        num_unpack_streams_in_folders.iter().enumerate()
                    {
                        if num_streams > 1 {
                            self.println_info(&format!(
                                "Folder {} substream sizes (first {} of {}):",
                                folder_idx,
                                num_streams - 1,
                                num_streams
                            ));
                            self.indent += 1;

                            for stream_idx in 0..(num_streams - 1) {
                                let size = self.read_encoded_uint64();
                                self.println_info(&format!(
                                    "Substream {}: {} bytes",
                                    stream_idx, size
                                ));
                                size_idx += 1;
                            }

                            self.println_info(&format!(
                                "Substream {}: (calculated from remaining)",
                                num_streams - 1
                            ));
                            self.indent -= 1;
                        }
                    }

                    if size_idx == 0 {
                        self.println_info("No sizes to read (all folders have <= 1 substream)");
                    }

                    self.indent -= 1;
                }
                K_CRC => {
                    self.println_info("SubStream CRCs:");
                    self.indent += 1;

                    // Calculate how many CRCs we need
                    // CRCs are needed for:
                    // - All substreams in folders with multiple substreams
                    // - Substreams in folders with single substream but no folder CRC
                    let mut crc_count = 0u64;
                    let mut crc_needed_for = Vec::new();

                    for (folder_idx, &num_streams) in
                        num_unpack_streams_in_folders.iter().enumerate()
                    {
                        if num_streams > 1 {
                            // Multiple substreams - always need CRCs
                            crc_count += num_streams;
                            for i in 0..num_streams {
                                crc_needed_for.push((folder_idx, i));
                            }
                        } else if num_streams == 1 {
                            // Single substream - need CRC if folder doesn't have one
                            let folder_has_crc = streams_info
                                .and_then(|si| si.folder_has_crc.get(folder_idx))
                                .copied()
                                .unwrap_or(false);

                            if !folder_has_crc {
                                crc_count += 1;
                                crc_needed_for.push((folder_idx, 0));
                            }
                        }
                    }

                    self.println_info(&format!("Expected CRC count: {}", crc_count));

                    // Only read CRCs if we actually expect any
                    if crc_count > 0 {
                        let all_defined = self.read_u8();

                        if all_defined != 0 {
                            self.println_info("All CRCs defined");

                            // Read all CRCs
                            for &(folder_idx, stream_idx) in &crc_needed_for {
                                let crc = self.read_u32();
                                self.println_info(&format!(
                                    "Folder {} Substream {} CRC: 0x{:08X}",
                                    folder_idx, stream_idx, crc
                                ));
                            }
                        } else {
                            self.println_info("CRCs selectively defined (reading bit vector)");

                            // Read bit vector
                            let bytes_needed = crc_count.div_ceil(8) as usize;
                            let bit_vector = self.read_bytes(bytes_needed);

                            self.println_info(&format!(
                                "Bit vector ({} bytes): {:02X?}",
                                bytes_needed, bit_vector
                            ));

                            // Read CRCs for defined entries
                            for (idx, &(folder_idx, stream_idx)) in
                                crc_needed_for.iter().enumerate()
                            {
                                let byte_index = idx / 8;
                                let bit_index = idx % 8;
                                let is_defined = (bit_vector[byte_index] & (1 << bit_index)) != 0;

                                if is_defined {
                                    let crc = self.read_u32();
                                    self.println_info(&format!(
                                        "Folder {} Substream {} CRC: 0x{:08X}",
                                        folder_idx, stream_idx, crc
                                    ));
                                } else {
                                    self.println_info(&format!(
                                        "Folder {} Substream {} CRC: undefined",
                                        folder_idx, stream_idx
                                    ));
                                }
                            }
                        }
                    } else {
                        self.println_info("No substream CRCs needed (folder CRCs already defined)");
                    }

                    self.indent -= 1;
                }
                _ => {
                    self.println_info(&format!(
                        "Unknown substreams property: 0x{:02X}",
                        property_type
                    ));
                }
            }
        }
    }

    fn parse_files_info(&mut self) {
        let num_files = self.read_encoded_uint64();
        self.println_info(&format!("NumFiles: {}", num_files));

        loop {
            let property_type = self.read_u8();

            if property_type == 0 {
                self.println_info("End of files info");
                break;
            }

            let size = self.read_encoded_uint64();
            self.println_info(&format!(
                "Property: {} (0x{:02X}), Size: {} bytes",
                property_id_to_string(property_type),
                property_type,
                size
            ));

            // For demonstration, just skip the property data
            // In a full implementation, you would parse each property type
            self.file.seek(SeekFrom::Current(size as i64)).unwrap();
        }
    }

    fn run(&mut self) {
        println!("7-Zip Archive Structure Debug Utility");
        println!("=====================================");
        println!();

        self.parse_signature_header();

        println!();
        println!("=====================================");
        println!("End of structure analysis");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <7z-file>", args[0]);
        std::process::exit(1);
    }

    let mut reader = SevenZipReader::new(&args[1]);
    reader.run();
}
