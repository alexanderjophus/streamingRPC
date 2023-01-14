# StreamingRPC

## Introduction

This is an exercise in building a streaming RPC server.

For both Go and Rust 🎉

## Usage

```bash
go run cmd/server/main.go
```
or
```bash
cargo run --bin server
```

Then using [Evans](https://github.com/ktr0731/evans);

On as many terminals as you want

```bash
evans --proto greet/v1/greet.proto --port 8080
call GreetStream
```

On another terminal

```bash
evans --proto greet/v1/greet.proto --port 8080
call Greet
```

and enter a user name.

### Frontend

```bash
cd web
npm run dev
```

Magic.

## Hacking

This is written using connect.build

```bash
buf generate
```

- I'm still working on fixing the import path for the go side, it shouldn't be relative
- Web seems to be lagging behind, I'm not sure why. Essentially when second message is sent, first is received.
  - https://github.com/bufbuild/connect-web/issues/233