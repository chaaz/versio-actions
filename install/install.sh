apt-get update && apt-get install -y curl

latest_url=`\
  curl -sL https://api.github.com/repos/chaaz/versio/releases/latest \
  | jq -r '.assets[] | select(.browser_download_url | contains("linux-gnu")) | .browser_download_url'`

curl -L $latest_url -o $HOME/bin/versio
chmod a+x $HOME/bin/versio

# Also install rq
curl -LSfs https://japaric.github.io/trust/install.sh \
    | sh -s -- --git dflemstr/rq --target x86_64-unknown-linux-gnu \
    --to $HOME/bin


# non-jq technique:
#
# latest_url=`\
#   curl -s https://api.github.com/repos/chaaz/versio/releases/latest \
#   | grep 'browser_download_url.*unknown-linux-gnu' \
#   | cut -d : -f 2,3 \
#   | tr -d \" \
#   | awk '{ print $1 }'`
