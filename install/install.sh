release_identifier=$INPUT_VERSION

apt-get update && apt-get install -y curl

versio_binary_url=`\
  curl -sL https://api.github.com/repos/chaaz/versio/releases/${release_identifier} \
  | jq -r '.assets[] | select(.browser_download_url | contains("linux-gnu")) | .browser_download_url'`


curl -L $versio_binary_url -o $HOME/bin/versio
chmod a+x $HOME/bin/versio

# Also install rq
curl -LSfs https://japaric.github.io/trust/install.sh \
    | sh -s -- --git dflemstr/rq --target x86_64-unknown-linux-gnu \
    --tag 1.0.2 --to $HOME/bin


# non-jq technique:
#
# latest_url=`\
#   curl -s https://api.github.com/repos/chaaz/versio/releases/latest \
#   | grep 'browser_download_url.*unknown-linux-gnu' \
#   | cut -d : -f 2,3 \
#   | tr -d \" \
#   | awk '{ print $1 }'`
