echo "export PATH=/sbin:/bin:/usr/bin:/usr/sbin:/usr/local/sbin:/usr/local/bin" >> /root/.bashrc
source /root/.bashrc

apt update
apt upgrade -y
apt install wget -y


wget -q https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
dpkg -i packages-microsoft-prod.deb
apt update
apt install sysmonforlinux -y
sysmon -accepteula -i