# lnurl

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/edouardparis/rust-lnurl/blob/master/LICENSE)
[![lnurl on crates.io](https://img.shields.io/crates/v/lnurl.svg)](https://crates.io/crates/lnurl)
[![lnurl on docs.rs](https://docs.rs/lnurl/badge.svg)](https://docs.rs/lnurl)

## Documentation

_Readings about **lnurl**_


* [The spec](https://github.com/btcontract/lnurl-rfc/blob/master/spec.md) &ndash; **lnurl** spec, by [Anton Kumaigorodski](https://twitter.com/akumaigorodski).
* [An introduction to lnurl](https://telegra.ph/lnurl-a-protocol-for-seamless-interaction-between-services-and-Lightning-wallets-08-19) &ndash; An article introducing the various types of **lnurl**'s, by [fiatjaf](https://twitter.com/fiatjaf).

## Progress

- [x] lnurl-withdraw
- [ ] lnurl-auth
- [ ] lnurl-pay
- [ ] lnurl-channel

## Usage

You will certainly need some crates like:
```toml
bech32 = "0.7.1"
lightning-invoice = "0.2.0"
serde = { version = "1.0.93", features =["derive"]}
serde_json = "1.0.39"
```

Create a bech32 QRCode:
```rust
use bech32::ToBase32;
use image::Luma;
use qrcode::QrCode;

pub fn create_lnurl_qrcode(url: &str, path: &str) {
    let encoded = bech32::encode("lnurl", url.as_bytes().to_base32()).unwrap();
    let code = QrCode::new(encoded.to_string()).unwrap();
    let image = code.render::<Luma<u8>>().build();
    image.save(path).unwrap();
}
```

Use serde_json to encode your LNRUL object in the HTTP response body
of your server.

```rust
if let Err(_) = invoice.parse::<lightning_invoice::SignedRawInvoice>() {
    let res = serde_json::to_string(
        &lnurl::Response::Error{reason: "your invoice is wrong".to_string()}
    ).unwrap();
    return Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(res)).unwrap())
}
```

See [lnurl-examples](https://github.com/edouardparis/lnurl-examples)

