# Registry Entry Collector
This tool gets the entry IDs of each synchronized registry and every vanilla tag from the vanilla data pack,
and writes them in `sandstone/src/protocol/game/info/registry/vanilla_registries.rs`.

The server uses these IDs when the client and the server share the `minecraft:core` known pack. The server
must list every entry, but it can leave out the entry data. The client then uses its own copy of the data.
Tags are never taken from a known pack, so the server sends all vanilla tags as numeric IDs.

Requirements:
- The vanilla client jar of the target version (for example `.minecraft/versions/26.3/26.3.jar`).
- The `registries.json` report of the same version. Make it with the `block_states` collector
  (`cargo run -p block_states -- report 26.3`). It gives the numeric IDs of the built-in registries, such as
  `block` and `item`.
- Python 3, to extract the jar. (In Git Bash on Windows, `unzip` made empty folders and extracted no files.)

## Steps
1. Extract the vanilla data pack from the client jar into `work/<version>/`:
   ```
   python -c "import zipfile,sys; z=zipfile.ZipFile(sys.argv[1]); z.extractall(sys.argv[2], [m for m in z.namelist() if m.startswith('data/minecraft/') and not m.endswith('/')])" <path to 26.3.jar> data_collectors/registry_entries/work/26.3
   ```
2. Write the entry lists and tags:
   ```
   cargo run -p registry_entries -- generate data_collectors/registry_entries/work/26.3/data/minecraft data_collectors/block_states/work/26.3/generated/reports/registries.json 26.3
   ```

Entries are sorted by ID. This is the order that the vanilla server uses, and the order sets the numeric ID
of each entry. The `work/` folder is not committed.

## Update instructions
If a version adds or removes a synchronized registry, change `SYNCED_REGISTRIES` in `src/main.rs`.
The list is in the [Registries page](https://minecraft.wiki/w/Java_Edition_protocol/Registries) on the wiki.
