# Yamaha Server

This project consists of two separate projects: this server and a UI.
The currently supported UI is developed by YONN2222 and can be found here: UI.

What does this project do?

This project communicates with the Yamaha **Tio1608-D Stagebox** via TCP and allows you to control the channel faders, phantom power, and mute states.

Technically, the project is also designed to support the *Yamaha TF1* using a separate mode. However, this feature is currently broken and still a work in progress.

The server is designed to run independently from the UI. This means that you can use the provided UI, create your own UI, or communicate with the server using your own client.

## Building

You can build the project using Cargo:
~~~
cargo build
~~~
The compiled executable can be found in:
~~~
./target/debug/
~~~
For a release build, you can use:
~~~
cargo build --release
~~~
The resulting executable will then be located in:
~~~
./target/release/
~~~
## Usage

You can either build the project as described above or run it directly using Cargo:
~~~
cargo run -- --address 0.0.0.0:8081
~~~
>The --address argument defines the address and port on which the server’s WebSocket can be reached. UI clients, such as the Yamaha Fader Web UI, can then connect to the server.

Instructions for installing and running the UI can be found in the UI project’s repository.

Since the server is standalone, it does not require a specific UI and can also be used with custom clients.
