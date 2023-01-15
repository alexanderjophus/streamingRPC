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
