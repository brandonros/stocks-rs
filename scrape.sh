#!/bin/bash
DATE=$1
SYMBOL=$2
RESOLUTION=$3
FROM_TIMESTAMP=$(date -j -f "%Y-%m-%d %I:%M:%S %p" "$DATE 04:00:00 AM" "+%s")
FROM_TIMESTAMP=$((FROM_TIMESTAMP * 1000))
TO_TIMESTAMP=$((FROM_TIMESTAMP + 57600000)) # 60 * 16 hours = 57600 seconds = 8pm
LIMIT=1000
curl "https://api.polygon.io/v2/aggs/ticker/$SYMBOL/range/$RESOLUTION/minute/$FROM_TIMESTAMP/$TO_TIMESTAMP?adjusted=true&sort=asc&limit=$LIMIT&apiKey=$POLYGON_API_KEY" -o "./data/polygon-$SYMBOL-$RESOLUTION-$FROM_TIMESTAMP-$TO_TIMESTAMP.json"
# TODO: jq check output to make sure API didn't return an error
