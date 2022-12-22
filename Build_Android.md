## How to build for android

As for now, rust version is 1.66, and the "gcc not found" issue on android still is not resolved.
To work around about this issue, there are two ways:

* use a ndk older than r23c
* rename libunwind.a to libgcc.a in ndk