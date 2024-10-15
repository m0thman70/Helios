target=target/release/atto
bin=/usr/bin

if [ $(id -u) == 0 ]; then
  cargo build --release
  chmod +x $target
  mv $target $bin
else
  echo "install.sh must be run with sudo. Doing nothing."
fi
