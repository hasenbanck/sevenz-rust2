use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Password(Vec<u8>);

impl Password {
    /// Creates a new [`Password`] from the given password string.
    ///
    /// Internally a password string is encoded as UTF-16.
    pub fn new(password: &str) -> Self {
        Self::from(password)
    }

    /// Creates a new [`Password`] from the given raw bytes.
    pub fn from_raw(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    pub fn empty() -> Self {
        Self(Default::default())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<[u8]> for Password {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for Password {
    fn from(s: &str) -> Self {
        let mut result = Vec::with_capacity(s.len() * 2);
        let utf16 = s.encode_utf16();
        for u in utf16 {
            let _ = result.write_u16::<LittleEndian>(u);
        }
        Self(result)
    }
}
