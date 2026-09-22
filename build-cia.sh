#!/bin/bash
rm -rf build-cia
mkdir build-cia

cargo 3ds build --release

bannertool makebanner -i cia/banner.png -a cia/audio.wav -o build-cia/banner.bnr

makerom -f cia -o build-cia/Presence3DS.cia \
  -rsf cia/app.rsf \
  -elf target/armv6k-nintendo-3ds/release/presence3ds-helper.elf \
  -icon target/armv6k-nintendo-3ds/release/presence3ds-helper.smdh \
  -banner build-cia/banner.bnr \
  -target t \
  -exefslogo