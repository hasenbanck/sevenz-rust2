use super::*;

impl BCJFilter {
    pub fn new_arm(start_pos: usize, encoder: bool) -> Self {
        Self {
            is_encoder: encoder,
            pos: start_pos + 8,
            prev_mask: 0,
            filter: Self::arm_code,
        }
    }

    pub fn new_arm_thumb(start_pos: usize, encoder: bool) -> Self {
        Self {
            is_encoder: encoder,
            pos: start_pos + 4,
            prev_mask: 0,
            filter: Self::arm_thumb_code,
        }
    }

    fn arm_code(&mut self, buf: &mut [u8]) -> usize {
        let len = buf.len();
        if len < 4 {
            return 0;
        }
        let end = len - 4;
        let mut i = 0;
        while i <= end {
            let b3 = buf[i + 3];

            if b3 == 0xEB {
                let b2 = buf[i + 2] as i32;
                let b1 = buf[i + 1] as i32;
                let b0 = buf[i] as i32;

                let src = ((b2 << 16) | (b1 << 8) | (b0)) << 2;
                let p = (self.pos + i) as i32;
                let dest = if self.is_encoder { src + p } else { src - p };
                let dest = dest >> 2;
                buf[i + 2] = ((dest >> 16) & 0xff) as u8;
                buf[i + 1] = ((dest >> 8) & 0xff) as u8;
                buf[i] = (dest & 0xff) as u8;
            }
            i += 4;
        }

        self.pos += i;
        i
    }

    fn arm_thumb_code(&mut self, buf: &mut [u8]) -> usize {
        let len = buf.len();
        if len < 4 {
            return 0;
        }
        let end = len - 4;

        let mut i = 0;
        while i <= end {
            let b1 = buf[i + 1] as i32;
            let b3 = buf[i + 3] as i32;

            if (b3 & 0xF8) == 0xF8 && (b1 & 0xF8) == 0xF0 {
                let b2 = buf[i + 2] as i32;
                let b0 = buf[i] as i32;

                let src =
                    ((b1 & 0x07) << 19) | ((b0 & 0xFF) << 11) | ((b3 & 0x07) << 8) | (b2 & 0xFF);
                let src = src << 1;

                let dest = if self.is_encoder {
                    src + (self.pos + i) as i32
                } else {
                    src - (self.pos + i) as i32
                };
                let dest = dest >> 1;
                buf[i + 1] = (0xF0 | ((dest >> 19) & 0x07)) as u8;
                buf[i] = (dest >> 11) as u8;
                buf[i + 3] = (0xf8 | ((dest >> 8) & 0x07)) as u8;
                buf[i + 2] = ((dest) & 0xff) as u8;
                i += 2;
            }
            i += 2;
        }

        self.pos += i;
        i
    }

    pub fn new_arm64(start_pos: usize, encoder: bool) -> Self {
        Self {
            is_encoder: encoder,
            pos: start_pos + 8,
            prev_mask: 0,
            filter: Self::arm64_code,
        }
    }

    fn arm64_code(&mut self, buf: &mut [u8]) -> usize {

        let len = buf.len();
        if len < 4 {
            return 0;
        }
        let end = len - 4;
        let mut i = 0;
        while i <= end {
            

                let b3 = buf[i + 3] as i32;
                let b2 = buf[i + 2] as i32;
                let b1 = buf[i + 1] as i32;
                let b0 = buf[i] as i32;

                let src =  (b3 << 24) + (b2 << 16) + (b1 << 8) + b0 ;

                let p = (self.pos + i) as i32;

                //BL
                if (src>>26) == 0x25 {
                    
                    let dest_adr = if self.is_encoder { src + (p>>2) } else { src - (p>>2) }; 

                    let dest = (dest_adr & 0x03ffffff) | (0x94 << 24);

                    buf[i + 3] = ((dest >> 24) & 0xff) as u8;
                    buf[i + 2] = ((dest >> 16) & 0xff) as u8;
                    buf[i + 1] = ((dest >> 8) & 0xff) as u8;
                    buf[i] = (dest & 0xff) as u8;

                }
                
                //ADRP
                if (src&(0x9f>>24))==0x90 {

                    let addr = ((src >>29)&3) | ((src>>3)&0x001FFFFC);

                    if 0==((addr.wrapping_add(0x00020000)) & 0x001C0000) {
                        let dest = (0x90 << 24) | (src & 0x1F);

                        let addr = if self.is_encoder { addr + (p>>12)} else { addr - (p>>12)};

                        let dest = dest | ((addr & 3) << 29);
                        let dest = dest | (addr & 0x0003FFFC) << 3;

                        let dest = dest | (0 - (dest & 0x00020000)) & 0x00E00000;

                        buf[i + 3] = ((dest >> 24) & 0xff) as u8;
                        buf[i + 2] = ((dest >> 16) & 0xff) as u8;
                        buf[i + 1] = ((dest >> 8) & 0xff) as u8;
                        buf[i] = (dest & 0xff) as u8;
                    }

                }
                
                i += 4;


        }
        self.pos += i;
        i
    }
}
