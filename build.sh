if [ ! -d ./bin ]; then
    mkdir ./bin
fi

cargo build --release --bins
cp -f ./target/release/quodlibet-status ./bin/quodlibet-status
cp -f ./target/release/quodlibet-volume ./bin/quodlibet-volume
cp -f ./target/release/pulse-status ./bin/pulse-status
cp -f ./target/release/nvidia-status ./bin/nvidia-status
