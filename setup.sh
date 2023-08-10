mkdir /root/Documents
mkdir /root/Downloads
echo "hoge huga" > /root/config.txt
echo "1234abc" > /root/Documents/document1.txt
echo "5678def" > /root/Documents/document2.txt

apt update
apt upgrade -y
apt install wget -y


wget -q https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
dpkg -i packages-microsoft-prod.deb
apt update
apt install sysmonforlinux -y
mount -t debugfs none /sys/kernel/debug

SYSMON_CONFIG="
<Sysmon schemaversion=\"4.81\">
    <HashAlgorithms>*</HashAlgorithms>
    <EventFiltering>
        <ProcessCreate onmatch=\"exclude\" />
        <FileCreateTime onmatch=\"exclude\" />
        <NetworkConnect onmatch=\"exclude\" />
        <ProcessTerminate onmatch=\"exclude\" />
        <DriverLoad onmatch=\"exclude\" />
        <ImageLoad onmatch=\"exclude\" />
        <CreateRemoteThread onmatch=\"exclude\" />
        <RawAccessRead onmatch=\"exclude\" />
        <ProcessAccess onmatch=\"exclude\" />
        <FileCreate onmatch=\"exclude\" />
        <RegistryEvent onmatch=\"exclude\" />
        <FileCreateStreamHash onmatch=\"exclude\" />
        <PipeEvent onmatch=\"exclude\" />
        <WmiEvent onmatch=\"exclude\" />
        <DnsQuery onmatch=\"exclude\" />
        <FileDelete onmatch=\"exclude\" />
        <ClipboardChange onmatch=\"exclude\" />
        <ProcessTampering onmatch=\"exclude\" />
        <FileDeleteDetected onmatch=\"exclude\" />
    </EventFiltering>
</Sysmon>
"

echo $SYSMON_CONFIG > config.xml
sysmon -accepteula -i config.xml