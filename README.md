# AMD GPU Module for Waybar CFFI

## Index
- [Building](#building)
  - [Target Platforms](#target-platforms)
  - [Dependencies](#dependencies)
    - Rust
      - [Fedora](#fedora)
      - [Arch](#arch)
      - [Debian](#debian)
    - libamd_smi.so
      - [Fedora](#fedora-1)
      - [Arch](#arch-1)
      - [Debian](#debian-1)
    - gtk3
      - [Fedora](#fedora-2)
      - [Arch](#arch-2)
      - [Debian](#debian-2)
  - Build
    - [Release](#release)
    - [Debug](#debug)
- [Usage](#usage)
  - [Waybar Config](#in-waybar-config)

## Building

### Target Platforms

- AMD64/x86_64

It only targets **AMD64** because the associating libraries don't cross compile to any other architecture :)

### Dependencies
- [AMD ROCm SMI](https://github.com/ROCm/amdsmi)

#### Packages that provides Rust
##### Fedora
 ```shell
yum install rust cargo -y
 ```
##### Arch
```shell
pacman -S rustup cargo --noconfirm
rustup default stable
```
##### Debian
```shell
apt install rustup build-essential; rustup default stable
```
##### Gentoo
C'mon guys you know how to install and search for packages

#### Package that provides libamd_smi.so
##### Fedora
 ```shell
yum install amdsmi -y
AMDSMI_LIB_DIR=$(find / -name "libamd_smi.so"); export AMDSMI_LIB_DIR=${AMDSMI_LIB_DIR::-14}
 ```
##### Arch
```shell
pacman -S amdsmi
```
##### Debian
```shell
apt install amd-smi
AMDSMI_LIB_DIR=$(find / -name "libamd_smi.so"); export AMDSMI_LIB_DIR=${AMDSMI_LIB_DIR::-14}
```

#### Package that provides gtk3
##### Fedora
 ```shell
yum install gtk3-devel -y
 ```
##### Arch
```shell
pacman -S gtk3 pkg-config --noconfirm
```
##### Debian
```shell
apt install libgtk-3-dev
```

### Build
#### Release
```shell
cargo b -r
```

#### Debug
```shell
cargo b
```
---
```shell
mkdir -p $HOME/.config/waybar/cffi/
mv target/release/libamdgpu.so $HOME/.config/waybar/cffi
```

## Usage
|Format Specifier|Description|
|---|---|
|{gpu_usage_percent}|GPU Usage as a percentage (no decimals)|
|{gpu_mem_total}|Total VRAM (formatted in IEC formatted bytes i.e 15.92GiB)|
|{gpu_mem_used}|Total VRAM Used (formatted in IEC formatted bytes i.e 527MiB)|
|{gpu_mem_used_percent}|Total VRAM Used as a percentage (no decimals)|
|{gpu_mem_free}|Total VRAM Unused (formatted in IEC formatted bytes i.e 14.46GiB)
|{gpu_usage}|Alias for `gpu_usage_percent`|
|{gpu_temp}|GPU Temperature measured in °C|

### In waybar config
Module Name: `cffi/gpu`

Module Spec
```json
  "cffi/gpu": {
    "module_path": "",
    "interval": 1, 
    "gpu_idx": 0,
    "format": ""
  },
```
|Key|Description|
|---|---|
|`"module_path"`|Path to the `libamdgpu.so` file| 
|`"interval"`|How often the module updates (sec)|
|`"gpu_idx"`|Index of the AMD GPUs in your system (usually this will be `0`)|
|`"format"`|Text outputted by the modules formats found [here](#usage)|
