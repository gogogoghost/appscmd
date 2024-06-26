## appscmd

This command line tool used on nokia 2780 flip (kaios 3.1) to sideload apps

Your device must be set selinux to permissive first. See [here](https://github.com/gogogoghost/root-nokia2780)

### Usage

[Download](https://github.com/gogogoghost/appscmd/releases) or build appscmd then push it to your device

```bash
adb push appscmd /data/local/tmp/
```

Then use install/install-pwa/list to manage apps on your device

```bash
# install a app
adb shell /data/local/tmp/appscmd install /data/local/tmp/application.zip

# install a pwa
adb shell /data/local/tmp/appscmd install-pwa https://xxx.com/manifest.webmanifest

# list apps
adb shell /data/local/tmp/appscmd list
```

### Build

Install rust and configure for target armv7-linux-androideabi

Then run build.sh
