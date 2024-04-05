use crate::Compact;
use alloy_primitives::{Address, Bytes, Log, LogData};
use bytes::BufMut;

/// Implement `Compact` for `LogData` and `Log`.
impl Compact for LogData {
    fn to_compact<B>(self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        let mut buffer = bytes::BytesMut::new();
        self.topics().to_vec().specialized_to_compact(&mut buffer);
        self.data.to_compact(&mut buffer);
        let total_length = buffer.len();
        buf.put(buffer);
        total_length
    }

    fn from_compact(mut buf: &[u8], _: usize) -> (Self, &[u8]) {
        let (topics, new_buf) = Vec::specialized_from_compact(buf, buf.len());
        buf = new_buf;
        let (data, buf) = Bytes::from_compact(buf, buf.len());
        let log_data = LogData::new_unchecked(topics, data);
        (log_data, buf)
    }
}

impl Compact for Log {
    fn to_compact<B>(self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        let mut buffer = bytes::BytesMut::new();
        self.address.to_compact(&mut buffer);
        self.data.to_compact(&mut buffer);
        let total_length = buffer.len();
        buf.put(buffer);
        total_length
    }

    fn from_compact(mut buf: &[u8], _: usize) -> (Self, &[u8]) {
        let (address, new_buf) = Address::from_compact(buf, buf.len());
        buf = new_buf;
        let (log_data, new_buf) = LogData::from_compact(buf, buf.len());
        buf = new_buf;
        let log = Log { address, data: log_data };
        (log, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::{Compact, Log};
    use proptest::proptest;

    proptest! {
        #[test]
        fn roundtrip(log: Log) {
            let mut buf = Vec::<u8>::new();
            let len = log.clone().to_compact(&mut buf);
            let (decoded, _) = Log::from_compact(&buf, len);
            assert_eq!(log, decoded);
        }
    }
}
