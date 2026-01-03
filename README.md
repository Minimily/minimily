# Minimily

## Config

### Session Secret Key

#### Entries

The `SESSION_SECRET_KEY` environment variable is used for session encryption.
It should be a random string of at least 64 bytes. To generate a proper 64 byte
key, use the following command:

```cmd
$ openssl rand -base64 48
```

48 bytes in base64 results in 64 characters.

## Heroku

### Buildpack

https://github.com/emk/heroku-buildpack-rust