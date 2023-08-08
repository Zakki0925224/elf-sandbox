# elf-sandbox

ELF sandbox

on ubuntu server 20.04:
```sh
echo "$(id -un) veth lxcbr0 10" | sudo tee /etc/lxc/lxc-usernet
mkdir -p ~/.config/lxc
cp /etc/lxc/default.conf ~/.config/lxc/default.conf

sudo usermod --add-subuids 100000-165536 $USER
sudo usermod --add-subgids 100000-165536 $USER

MS_UID="$(grep "$(id -un)" /etc/subuid  | cut -d : -f 2)"
ME_UID="$(grep "$(id -un)" /etc/subuid  | cut -d : -f 3)"
MS_GID="$(grep "$(id -un)" /etc/subgid  | cut -d : -f 2)"
ME_GID="$(grep "$(id -un)" /etc/subgid  | cut -d : -f 3)"
echo "lxc.idmap = u 0 $MS_UID $ME_UID" >> ~/.config/lxc/default.conf
echo "lxc.idmap = g 0 $MS_GID $ME_GID" >> ~/.config/lxc/default.conf

cargo run --setup-sh-path ./sysmon-setup.sh --target-elf-path <path> --timeout <secs>
```