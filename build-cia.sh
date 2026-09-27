#!/bin/bash
rm -rf build-cia
mkdir build-cia

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
MAJOR=$(echo $VERSION | cut -d. -f1)
MINOR=$(echo $VERSION | cut -d. -f2)
MICRO=$(echo $VERSION | cut -d. -f3)

cargo 3ds build --release

bannertool makebanner -i cia/banner.png -a cia/audio.wav -o build-cia/banner.bnr

makerom -f cia -o build-cia/presence3ds-helper.cia \
  -rsf cia/app.rsf \
  -elf target/armv6k-nintendo-3ds/release/presence3ds-helper.elf \
  -icon target/armv6k-nintendo-3ds/release/presence3ds-helper.smdh \
  -banner build-cia/banner.bnr \
  -target t \
  -exefslogo \
  -major $MAJOR -minor $MINOR -micro $MICRO -v
 