# Client Login Example
This is an example of a client that can connect to a Minecraft server, perform the login procedure, and remain connected.
It can handle all packets of an idle server, where it is the only connected client.
<br>*Theres a small chance that it might not be able to handle some packets it encounters because they haven't been implemented yet.*

Requirements:
- Minecraft server running on `localhost:25565` AKA `127.0.0.1:25565`
- Server should be running with online mode disabled (i.e. `online-mode=false` in `server.properties`)
- Server should disable compression (i.e. `compression-threshold=-1` in `server.properties`)