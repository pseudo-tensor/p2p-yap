#!/bin/bash
set -e

# 1. Create two isolated network namespaces
ip netns add peer1
ip netns add peer2

# 2. Create a virtual ethernet pair (a virtual cable connecting the two)
ip link add veth-p1 type veth peer name veth-p2

# 3. Move each end of the cable into its respective namespace
ip link set veth-p1 netns peer1
ip link set veth-p2 netns peer2

# 4. Assign IP addresses
ip netns exec peer1 ip addr add 10.0.0.1/24 dev veth-p1
ip netns exec peer2 ip addr add 10.0.0.2/24 dev veth-p2

# 5. Bring up the interfaces (including loopback)
ip netns exec peer1 ip link set dev veth-p1 up
ip netns exec peer1 ip link set dev lo up

ip netns exec peer2 ip link set dev veth-p2 up
ip netns exec peer2 ip link set dev lo up

echo "Sandbox ready! peer1: 10.0.0.1 | peer2: 10.0.0.2"
