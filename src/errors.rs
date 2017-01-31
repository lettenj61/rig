use std::io;
use url;

error_chain! {

    foreign_links {
        Io(io::Error);
        ParseUrl(url::ParseError);
    }

    errors {
        InvalidUrlFormat(s: String) {
            description("Invalid URL format")
            display("Invalid URL format: {}", s)
        }
        TomlDecodeFailure {
            description("Failured decoding Toml string")
            display("Failured decoding Toml string")
        }
    }
}
