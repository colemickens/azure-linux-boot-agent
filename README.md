# azure-linux-boot-agent
*a light-weight, safe, sane alternative to [`walinuxagent`](https://github.com/Azure/walinuxagent)*

## overview

asdfasdlf

## goals
* minimal agent, can be configured to ONLY complete Azure boot process
* optional features:
  * provision wheel user with `authorized_keys` based on `az vm create` data
  * seed /dev/random with hyper-v `OEM0` entropy
* pure Rust, no unsafe, no 100+MB python runtime needed
  * note, the internal error handling is not great yet, so take this claim lightly
* *NO DYNAMIC RUNTIME SCHENANIGANS*

## why

* link
* to
* dozen
* walinuxagent
* bugs

## support

link to donate page

## todo

* consider replacing serde with simple string matching or xpath or...?
* nix packaging - slim build further?
* udev rules appled in destination VM? how to verify easily? (gen2 VM considerations?)
* support ovf-env.xml instead of using IMDS for user/keydata?
* support ovf-env.xml for userdata (lol since IMDS can't...)

## todo (unrelated, but related, sort of)

* See if the VM can extract the MSI private key, is that what's pushed in?

## Future Plans

This will likely learn how to download the `userdata`:

* support Nix file as userdata?
* support pointing to an existing system closure?
* ideal scenario: we build a way to cache NARs in Azure and use an azure mirror
* that gets us to offline deploys

## Unplanned Plans

* Support Certificates
  * no. Use KV + MSI in IMDS (lol, how many years did it take?)
  * also, consider `nix-sops` which will soon support Azure

* Support Extensions <- no. I am not sufficiently motivated to care about non-NixOS Linuxes.
* Support Emergency Access <- no. Deploy a SSH CA or put your key on a yubikey and don't lose it.

* Support direct KV secret loading
  * no, use mozilla/sops

## tests

There is one acceptance/testing criteria, does this work as part of `flake-azure`?

So, it's status is reflective of our status: [![builds.sr.ht status](https://builds.sr.ht/~colemickens/flake-azure.svg)](https://builds.sr.ht/~colemickens/flake-azure?)

## manual testing

```shell
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder'
rust-musl-builder cargo build --release
rsync ./target/... to VM
```




#### License

Contact me if the license is an issue for you.


#### todo: fetch keys:

LOL, is this reliable enough to use at boot

```
curl -H Metadata:true --noproxy "*" "http://169.254.169.254/metadata/instance/compute/publicKeys?api-version=2019-08-01&format=json"
```

#### todo: seed entropy pool
cat /sys/firmware/acpi/tables/OEM0 > /dev/random
https://github.com/Azure/WALinuxAgent/issues/1500#issuecomment-659148981

follow-up https://github.com/Azure/WALinuxAgent/issues/1946



## wtf

Jul 19 12:46:08 localhost kernel: MDS CPU bug present and SMT on, data leak possible. See https://www.kernel.org/doc/html/latest/admin-guide/hw-vuln/mds.html for more details.
Jul 19 12:46:08 localhost kernel: TAA CPU bug present and SMT on, data leak possible. See https://www.kernel.org/doc/html/latest/admin-guide/hw-vuln/tsx_async_abort.html for more details.
Jul 19 12:46:08 localhost kernel:    #2   #3   #4   #5   #6   #7


# this is a pssible bug:
```rust
    let re = Regex::new(r"^/home/([a-zA-Z0-9]+)/.ssh/authorized_keys$").unwrap();
```