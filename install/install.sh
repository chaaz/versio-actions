apt-get update && apt-get install -y curl

# would be nice to use jq:
#
# latest_url=`\
#   curl -sL https://api.github.com/... \
#   | jq -r '.assets[] | select(.browser_download_url | contains("linux-gnu")) | .browser_download_url'

latest_url=`\
  curl -s https://api.github.com/repos/chaaz/versio/releases/latest \
  | grep 'browser_download_url.*unknown-linux-gnu' \
  | cut -d : -f 2,3 \
  | tr -d \" \
  | awk '{ print $1 }'`

curl -L $latest_url -o $HOME/bin/versio
chmod a+x $HOME/bin/versio
