use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S3Sequencer {
    bucket_name: String,
    object_key: String,
    sequencer: String,
}

impl S3Sequencer {
    pub fn new(bucket_name: &str, objcet_key: &str, sequencer: &str) -> Self {
        Self {
            bucket_name: bucket_name.to_owned(),
            object_key: objcet_key.to_owned(),
            sequencer: sequencer.to_owned(),
        }
    }
}

impl PartialEq for S3Sequencer {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other)
            .map_or(false, |o| o == std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for S3Sequencer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // バケット名が異なるシーケンサは比較できない
        if self.bucket_name != other.bucket_name {
            return None;
        }

        // オブジェクトキーが異なるシーケンサは比較できない
        if self.object_key != other.object_key {
            return None;
        }

        // 長い方に合わせて、短い方の末尾に0を追加して比較
        let max_len = std::cmp::max(self.sequencer.len(), other.sequencer.len());
        let self_sequencer = self
            .sequencer
            .chars()
            .chain(std::iter::repeat('0'))
            .take(max_len);
        let other_sequencer = other
            .sequencer
            .chars()
            .chain(std::iter::repeat('0'))
            .take(max_len);

        Some(self_sequencer.cmp(other_sequencer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_partial_cmp() {
        let bucket_name = "bucket";
        let object_key = "object";

        let s1 = S3Sequencer::new(bucket_name, object_key, "1");
        let s10 = S3Sequencer::new(bucket_name, object_key, "10");
        let s2 = S3Sequencer::new(bucket_name, object_key, "2");
        let sab = S3Sequencer::new(bucket_name, object_key, "ab");
        let sac = S3Sequencer::new(bucket_name, object_key, "ac");

        assert!(s1 == s10);
        assert!(s10 < s2);
        assert!(s1 < s2);
        assert!(s2 < sab);
        assert!(sab < sac);

        let other_bucket_name = "other_bucket";
        let other_object_key = "other_object";
        let other = S3Sequencer::new(other_bucket_name, other_object_key, "1");

        assert!(s1 != other);
    }
}
