# StreamingRPC

## Introduction

This is an exercise in building a streaming RPC server.

## Usage

```bash
go run cmd/server/main.go
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

## Hacking

This is written using connect.build

```bash
buf generate
```

(I'm still working on fixing the import path)