# Peer to Peer chatting cli in Rust

This serves close to zero purpose

## Setup:

1. Run the script scripts/netns_setup.sh with elevated permissions.
2. Use ``` sudo ip netns exec <peer_name> su - $USER -c "cd <path_to_proj_dir> && cargo run" ``` to create a peer for locally testing using ip netns namespaces. For a compiled executable, use ``` sudo ip netns exec <peer_name> su - $USER -c "cd <path_to_compiled_exe>" ``` instead.