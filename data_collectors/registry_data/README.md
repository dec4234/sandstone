# Registry Data Collector
This is a tool that collects registry data that is sent by the server to the client during the client
login process. Each registry packet will be saved to a json file in the `output` directory.

Requirements:
- (Only tested with Vanilla) Minecraft server running on `localhost:25565` AKA `127.0.0.1:25565`
- Server should be running with online mode disabled (i.e. `online-mode=false` in `server.properties`)
- Server should disable compression (i.e. `compression-threshold=-1` in `server.properties`)