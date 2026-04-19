# Minimily

## Config

### .env File

```cmd
$ cp .env.example .env
```

### Session Secret Key

#### Entries

The `SESSION_SECRET_KEY` environment variable is used for session encryption.
It should be a random string of at least 64 bytes. To generate a proper 64 byte
key, use the following command:

```cmd
$ openssl rand -base64 48
```

48 bytes in base64 results in 64 characters.

## Database Migration

```cmd
$ cargo clean
$ cargo run
```

The clean is necessary because the `migrate!` macro reads the files in the
migration folder and embeds them into the binary at the moment we compile the code.

## Heroku

### Buildpack

https://github.com/emk/heroku-buildpack-rust