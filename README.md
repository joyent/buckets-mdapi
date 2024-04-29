# manta-buckets-mdapi: The Manta buckets metadata API

This repository comprises a rust workspace with two different binary project and
a library project. The binaries are `buckets-mdapi` and `schema-manager`.

`buckets-mdapi` is a component of the manta buckets storage system that exposes
an API for interacting with Manta object metadata using the
[Fast](https:/github.com/TritonDataCenter/node-fast) protocol.

`schema-manager` is a tool that performs the necessary database configuration
to ensure the database is properly prepared for `buckets-mdapi`. Each time it
executes it only does the necessary work to ensure the database correctly
prepared. It will not overwrite or destroy existing structures in the database.

## Quick start

The following steps
    cp config.toml.dist config.toml
    vi config.toml
    cargo run --bin buckets_mdapi -- -c config.toml

## Configuration

There are a variety of configuration options to tailor the behavior to
particular environments. The configuration options are organized into categories
and each are described below.

### Logging

The logging configuration entries pertaining the bucket-mdapi's logging behavior.

* `level` - The logging level for buckets-mdapi to use. Possible values are `Critical`,
  `Error`, `Warning`, `Info`, `Debug`, and `Trace`. The default value is `Info`.

### Server

The configuration entries controlling the buckets-mdapi server behavior.

* `host` - The IP address buckets-mdapi should use to listen for incoming connections.
* `port` - The port number buckets-mdapi should listen on for incoming connections.

### Metrics

The configuration entries controlling the buckets-mdapi metrics server.

* `host` - The IP address buckets-mdapi should use to listen for metrics requests.
* `port` - The port number buckets-mdapi should listen on for incoming metrics request
  connections.

### Database

The database connection configuration entries.

* `user` - The database username.
* `database` - The name of the database to issue requests against.
* `application_name` - The name of the application to use when connecting to the
  database.
* `tls_mode` - The TLS connection mode. Valid values are `disable`, `allow`,
  `prefer`, `require`, `verify-ca`, and `verify-full`. See the [postgres
  client documentation](https://www.postgresql.org/docs/current/libpq-ssl.html) on SSL support for more details about the meaning of
  these options. The default value is `disable`
* `certificate` - The optional path to a TLS certificate file when enabling TLS
  connections via the `tls_mode` configuration option.

### Cueball

The database connection pool configuration entries.

* `max_connections` - The maximum number of connections in the postgres
  connection pool. The default value is 64 connections
* `claim_timeout` - The time in milliseconds that a request to claim a
  connection from the cueball connection pool should wait before returning an
  error. The default is 500 ms.
* `rebalancer_action_delay` - The time in milliseconds to wait prior to
  rebalancing the connection pool when a notification is received from the
  resolver regarding a change in the service topology. For the case of buckets-mdapi
  using the postgres primary resolver this delay should not be very high. The
  default value is 20 ms.

### Tokio

Tokio provides the runtime for buckets-mdapi and these are configuration options to
control the behavior of the Tokio runtime.

* `core_threads` - The maximum number of worker threads for the Tokio Runtime's
  thread pool. This must be a number between 1 and 32,768 though it is advised
  to keep this value on the smaller side. The default value is the number of
  logical cores available to the system.
* `blocking_threads` - The maximum number of concurrent blocking sections in the
  Runtime's thread pool. When the maximum concurrent blocking calls is reached,
  any further calls to blocking will return `NotReady` and the task is notified
  once previously in-flight calls to blocking return. This must be a number
  between 1 and 32,768 though it is advised to keep this value on the smaller
  side. The default value is 200.
* `thread_keep_alive` - The worker thread keep alive duration for threads in the
  Tokio Runtime's thread pool. If set, a worker thread will wait for up to the
  specified duration (in seconds) for work, at which point the thread will
  shutdown. When work becomes available, a new thread will eventually be spawned
  to replace the one that shut down. When the value is `None` (*i.e.* It is
  omitted from the configuration file), the thread will wait for work forever. The default value is `None`.
* `thread_stack_size` - The stack size (in bytes) for worker threads. The
  default is 2 MiB.
* `thread_name_prefix` - The name prefix of threads spawned by the Tokio
  Runtime's thread pool. The default is `buckets-mdapi-worker-`.

## Database schema setup and migrations

The `schema-manager` binary is included in this repo as a tool to create and
configure the expected database schemas, tables, and other necssary aspects of
the database.

The details of the database setup may change over time along with the code and
so the `schema-manager` tool includes the ability to add database
migrations. More details about database migrations can be found [here](./migrations/MIGRATIONS.md).

## Testing

The tests can be run with:

```
cargo test
```

There are quickcheck tests and also some functional tests for the RPC handlers.

The functional tests require that postgresql is installed as well as
[`pg_tmp`](http://eradman.com/ephemeralpg/). The test uses `pg_tmp` to create
and configure a temporary postgres database and once the test has completed the
temporary database is removed within a few seconds. The tests use the same code
used by the `schema-manager` to prepare the database for use by the test suite.
