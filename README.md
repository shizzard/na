## Setup

First, boot up the Postgres container:

```
docker compose up -d
```

Second, run diesel migrations (requires the diesel binary to be intalled, but I
bet it already is):

```
diesel migration run
```

Third, maybe alter the service configuration.

One way to alter the configuration is to edit the `config/default.toml` file.

Another way is to use something like [direnv](https://direnv.net/). The example
`.envrc` file is located at `.envrc.example`.

At last, build and run the service:

```
cargo run
```

```
cargo test
```
