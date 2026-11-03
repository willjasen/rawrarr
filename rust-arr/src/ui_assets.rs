//! UI assets are compiled into the application image and served from that
//! image's source revision. Container volumes must never shadow these assets.

pub async fn text(_path: &'static str, embedded: &'static str) -> String {
    embedded.to_owned()
}

pub async fn bytes(_path: &'static str, embedded: &'static [u8]) -> Vec<u8> {
    embedded.to_vec()
}

#[cfg(test)]
mod tests {
    use super::{bytes, text};

    #[tokio::test]
    async fn always_serves_embedded_assets_regardless_of_bundle_path() {
        assert_eq!(
            text("/ui/current/page.html", "from image").await,
            "from image"
        );
        assert_eq!(
            bytes("/ui/current/asset.bin", b"from image").await,
            b"from image"
        );
    }
}
