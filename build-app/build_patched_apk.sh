# nix shell nixpkgs#apksigner nixpkgs#apktool
set -euo pipefail

rm -rf tmp
apktool d source-package.apk -o tmp --no-src
mkdir tmp/res/xml
cp network_security_config.xml tmp/res/xml/network_security_config.xml
sed -i -e 's/com.bethsoft.blade/fr.mariusdavid.blade/g' tmp/AndroidManifest.xml
sed -i -e 's#android:theme="@style/UnityThemeSelector"#android:theme="@style/UnityThemeSelector" android:networkSecurityConfig="@xml/network_security_config"#g' tmp/AndroidManifest.xml
sed -i -e 's#<string name="app_name">Blades</string>#<string name="app_name">custom Blades</string>#g' tmp/res/values/strings.xml
apktool b tmp/ -o patched.apk --use-aapt1


echo "signing"
apksigner sign --ks keys.keystore --ks-key-alias mytestkey --ks-pass pass:android patched.apk
#apksigner sign --key testkey.pk8 --cert testkey.x509.pem patched.apk
echo "verifying"
apksigner verify patched.apk
