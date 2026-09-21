# Yamaha Server
This Project consist of two Projects, this project and the UI. The only currently supported UI is written by [**YONN2222**](https://github.com/YONN2222) and is located here [**UI**](https://github.com/YONN2222).

## What does this Project do?
This Project communicates over TCP with the Tio1608-D Stagebox from Yamaha to control the DB slider, Phantom Power and Mute of the Channels. Technicly this Project is also designed to do the same for the TF1 using a seperat mode but this feature is currently Broken and work in progress.

## Building
~~~
cargo build
~~~
the compiled executable is located at
~~~
./target/debug/
~~~

## Usage
You can either build it as explained in the Building step or you can run it directly with
~~~
cargo run —- —-address 0.0.0.0:8081
~~~
Here the argument address is used to define where the WebSocket should be reacheable so UI Clients like the one from [**UI**](https://github.com/YONN2222) can connect and communicate with the Server.
