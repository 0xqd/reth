use crate::{Address, Bloom, Bytes, B256};
use alloy_primitives::Log as AlloyLog;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use reth_codecs::{main_codec, Compact};

/// Ethereum Log
#[main_codec(rlp)]
#[derive(Clone, Debug, PartialEq, Eq, RlpDecodable, RlpEncodable, Default)]
pub struct Log {
    /// Contract that emitted this log.
    pub address: Address,
    /// Topics of the log. The number of logs depend on what `LOG` opcode is used.
    #[cfg_attr(
        any(test, feature = "arbitrary"),
        proptest(
            strategy = "proptest::collection::vec(proptest::arbitrary::any::<B256>(), 0..=5)"
        )
    )]
    pub topics: Vec<B256>,
    /// Arbitrary length data.
    pub data: Bytes,
}

impl From<AlloyLog> for Log {
    fn from(mut log: AlloyLog) -> Self {
        Self {
            address: log.address,
            topics: std::mem::take(log.data.topics_mut_unchecked()),
            data: log.data.data,
        }
    }
}

impl From<Log> for AlloyLog {
    fn from(log: Log) -> AlloyLog {
        AlloyLog::new_unchecked(log.address, log.topics, log.data)
    }
}

/// Calculate receipt logs bloom.
pub fn logs_bloom<'a, It>(logs: It) -> Bloom
where
    It: IntoIterator<Item = &'a Log>,
{
    let mut bloom = Bloom::ZERO;
    for log in logs {
        bloom.m3_2048(log.address.as_slice());
        for topic in &log.topics {
            bloom.m3_2048(topic.as_slice());
        }
    }
    bloom
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::*;
    use alloy_primitives::LogData;
    use proptest::proptest;
    use serde::Serialize;
    use serde_json::{to_writer_pretty};

    #[derive(Serialize)]
    struct JsonOutput {
        topics: Vec<B256>,
        address: Address,
        data: Bytes,
        decoded: Vec<u8>
    }

    proptest! {
        #[test]
        fn roundtrip(log: Log) {
            let mut buf = Vec::<u8>::new();
            log.clone().to_compact(&mut buf);
            let (decoded, _) = Log::from_compact(&buf, buf.len());

            // create  file /tmp/log_test.json, create or append
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/log_test.json")
                .expect("Unable to open file");

            let json = JsonOutput {
                topics: log.topics.clone(),
                address: log.address.clone(),
                data: log.data.clone(),
                decoded: buf
            };
            to_writer_pretty(&mut file, &json).expect("Unable to write to file");

            assert_eq!(log, decoded);
        }
    }

}