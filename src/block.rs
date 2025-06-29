// In the 7zip specification this is called "folder". But since in the UI of 7zip they are called
// "block" we chose to also call them under that name.
#[derive(Debug, Default, Clone)]
pub struct Block {
    pub coders: Vec<Coder>,
    pub has_crc: bool,
    pub crc: u64,
    pub(crate) total_input_streams: usize,
    pub(crate) total_output_streams: usize,
    pub(crate) bind_pairs: Vec<BindPair>,
    pub(crate) packed_streams: Vec<u64>,
    pub(crate) unpack_sizes: Vec<u64>,
    pub(crate) num_unpack_sub_streams: usize,
}

impl Block {
    pub(crate) fn find_bind_pair_for_in_stream(&self, index: usize) -> Option<usize> {
        let index = index as u64;
        (0..self.bind_pairs.len()).find(|&i| self.bind_pairs[i].in_index == index)
    }

    pub(crate) fn find_bind_pair_for_out_stream(&self, index: usize) -> Option<usize> {
        let index = index as u64;
        (0..self.bind_pairs.len()).find(|&i| self.bind_pairs[i].out_index == index)
    }

    pub fn get_unpack_size(&self) -> u64 {
        if self.total_output_streams == 0 {
            return 0;
        }
        for i in (0..self.total_output_streams).rev() {
            if self.find_bind_pair_for_out_stream(i).is_none() {
                return self.unpack_sizes[i];
            }
        }
        0
    }

    pub fn get_unpack_size_for_coder(&self, coder: &Coder) -> u64 {
        for i in 0..self.coders.len() {
            if std::ptr::eq(&self.coders[i], coder) {
                return self.unpack_sizes[i];
            }
        }
        0
    }

    pub fn get_unpack_size_at_index(&self, index: usize) -> u64 {
        self.unpack_sizes.get(index).cloned().unwrap_or_default()
    }

    pub fn ordered_coder_iter(&self) -> OrderedCoderIter {
        OrderedCoderIter::new(self)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Coder {
    encoder_method_id: [u8; 0xF],
    pub(crate) id_size: usize,
    pub(crate) num_in_streams: u64,
    pub(crate) num_out_streams: u64,
    pub(crate) properties: Vec<u8>,
}

impl Coder {
    pub fn encoder_method_id(&self) -> &[u8] {
        &self.encoder_method_id[0..self.id_size]
    }

    pub(crate) fn decompression_method_id_mut(&mut self) -> &mut [u8] {
        &mut self.encoder_method_id[0..self.id_size]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BindPair {
    pub(crate) in_index: u64,
    pub(crate) out_index: u64,
}

pub struct OrderedCoderIter<'a> {
    block: &'a Block,
    current: Option<u64>,
}

impl<'a> OrderedCoderIter<'a> {
    fn new(block: &'a Block) -> Self {
        let current = block.packed_streams.first().copied();
        Self { block, current }
    }
}

impl<'a> Iterator for OrderedCoderIter<'a> {
    type Item = (usize, &'a Coder);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.current {
            self.current = if let Some(pair) = self.block.find_bind_pair_for_out_stream(i as usize)
            {
                Some(self.block.bind_pairs[pair].in_index)
            } else {
                None
            };
            self.block
                .coders
                .get(i as usize)
                .map(|item| (i as usize, item))
        } else {
            None
        }
    }
}
