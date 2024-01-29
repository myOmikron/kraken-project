#!/usr/bin/env bash
set -efu -o pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 [machine name] [vagrant box ID]"
    exit 1
fi

MACHINE_NAME="$1"
BOX_ID="$2"

function cleanup {
  set +efu +o pipefail
  vagrant destroy -f 1>&2
  rm -f Vagrantfile
}
trap cleanup EXIT

cat > Vagrantfile <<- EOS
Vagrant.configure("2") do |config|
  config.nfs.functional = false
  config.vm.provider "libvirt" do |v|
    v.default_prefix = "$MACHINE_NAME"
    v.memory = 2048
    v.cpus = 2
  end
  config.vm.define "$MACHINE_NAME", primary: true do |os|
    os.vm.hostname = "$MACHINE_NAME"
    os.vm.box = "$BOX_ID"
  end
end
EOS
vagrant up 1>&2
echo "machine is up at $(date), running SSH:" >&2
echo "SSH: $(echo "" | vagrant ssh -- -oBatchMode=true -v exit 2>&1 | grep -F 'remote software version')"
