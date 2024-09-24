# adbear

This is an attempt to alleviate a personal pain point of mine where I have to keep re-pairing my devices and the usual dance of obtaining pairing codes and typing in multiple ADB commands was growing quite wearisome.
It is inspired by the [ADB-QR](https://github.com/aakash-pamnani/ADB-QR) VSCode extension and has also been a reference implementation alongside the official [ADB Wifi](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/docs/dev/adb_wifi.md) architecture docs.

## Status

As of now the CLI works _most_ of the time but deals with any unexpected situations rather poorly. Some logging infrastructure also needs to be built up to ease debugging as well as do a better job of reporting things to users.

## Licensing

Dual licensed under Apache 2.0 or MIT at your option.
