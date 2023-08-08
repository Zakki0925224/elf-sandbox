sudo apt update
sudo apt upgrade -y
sudo apt install wget -y


wget -q https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
sudo dpkg -i packages-microsoft-prod.deb
sudo apt update
sudo apt install sysmonforlinux -y
sudo mount -t debugfs none /sys/kernel/debug
sudo sysmon -accepteula -i