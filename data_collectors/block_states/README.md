# Block State Collector
This tool gets block state IDs from the vanilla Minecraft server and writes them as Rust constants in
`sandstone/src/protocol/game/world/block_states.rs`.

Requirements:
- `java` on `PATH`. The Java version must be one that the server jar can run on.
- Network access to `piston-meta.mojang.com` and `piston-data.mojang.com` for the first run of a version.

## Steps
1. Make the report. This downloads the server jar into `work/<version>/` and runs the vanilla data generator:
   ```
   cargo run -p block_states -- report 26.3
   ```
   The report is `work/26.3/generated/reports/blocks.json`.
2. Write the constants from the report:
   ```
   cargo run -p block_states -- generate data_collectors/block_states/work/26.3/generated/reports/blocks.json 26.3
   ```

Each constant is the default state ID of one block (for example `GRASS_BLOCK` is `grass_block[snowy=false]`).

The same command also reads `registries.json` from the same folder and writes
`sandstone/src/protocol/game/world/block_state_table.rs`. That file has every block with its properties and
state ID range, and the block entity types in protocol ID order. The Anvil module uses it to change block state
names into state IDs. The tool checks the state order against every state in the report and stops if it changed.
The `work/` folder is not committed.

## Update instructions
Replace server.jar in the work/ directory with the server jar you want to target.
