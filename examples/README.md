# lnurl_auth

usage:

* `ngrok http 8383`
* `SERVICE_URL=https://xxx.ngrok.io cargo run --example lnurl_auth --features="service"`

then:

* get qrcode: `localhost:8383/login`
* list connected users: `localhost:8383/users`
