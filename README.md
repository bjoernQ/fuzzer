# Fuzzer

This uses `RustSmith` to generate random code and then checks it on ESP32 vs ESP32-C3 on Wokwi.

If there are possible candidates for mis-compilation it saves the generated code in the `findings` folder.

This is currently meant to be run on Windows but should be easy to adapt to run on Linux.

## Setup and Run

Make sure you have a JVM on the path. (`RustSmith` is written in Kotlin)

Then set environment variables like this
```
# Wokwi Token
export WOKWI_CLI_TOKEN=<WOKWI TOKEN>
```

Then run `cargo run` here.

It will often crash currently (because of missing / bad error handling).... just run it again.
