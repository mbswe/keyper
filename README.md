# Keyper

## Overview

A simple in-memory key-value store server, similar to a basic Redis clone. It supports various commands to set, get... The server also features a journaling system to log all modifying transactions for recovery purposes.

## Running the Server

To run the server, use the following command:

`cargo run`

By default, the server listens on `0.0.0.0:3344` but this can be configured in `config.toml`.

## Supported Commands

-   **SET key value [update]**: Sets a value for a key. If `update` is `true`, existing keys will be updated. If `update` is `false` or omitted, the command will error if the key already exists.
-   **GET key**: Retrieves the value for a key.
-   **DELETE key**: Removes a key from the store.
-   **INCR key**: Increments the integer value of a key.
-   **DECR key**: Decrements the integer value of a key.
-   **MSET key1 value1 key2 value2 ... keyN valueN**: Sets multiple key-value pairs.
-   **MGET key1 key2 ... keyN**: Retrieves values for multiple keys.
-   **CHECK key**: Checks if a key exists in the store.
-   **FLUSH**: Clears all data from the store.
-   **KEYS**: Lists all keys stored in the database.

## Journaling

The server logs all write operations (SET, DELETE, INCR, DECR, FLUSH, MSET) to a journal file (`journal.log`). Upon restart, the server replays the journal to restore its state.

## Error Handling

The server implements basic error handling and command validation. Invalid or malformed commands will result in an error message.


