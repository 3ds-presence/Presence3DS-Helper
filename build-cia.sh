#!/bin/bash
rm -rf build-cia
mkdir build-cia

cargo 3ds build --release

bannertool makebanner -i cia/banner.png -a cia/audio.wav -o build-cia/banner.bnr

makerom -f cia -o build-cia/Presence3DS.cia \
  -rsf cia/app.rsf \
  -elf target/armv6k-nintendo-3ds/release/Presence3DS_Helper.elf \
  -icon target/armv6k-nintendo-3ds/release/Presence3DS_Helper.smdh \
  -banner build-cia/banner.bnr \
  -target t \
  -exefslogo