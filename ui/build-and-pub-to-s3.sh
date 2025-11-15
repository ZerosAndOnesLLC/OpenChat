#!/bin/bash
set -e

#BUCKET="s3://identity.zerosandones.us"
BUCKET="s3://0n1-openchat-prod"

# CloudFront distribution ID for titanium-vault.com
DISTRIBUTION_ID="ESVEXFK33MU09"

echo "Running npm run build..."
npm run build

echo "Removing all files from $BUCKET..."
aws s3 rm "$BUCKET" --recursive --profile prod

echo "Copying ./out to $BUCKET..."
aws s3 cp ./out "$BUCKET" --recursive --profile prod

echo "Creating CloudFront invalidation..."
if [ "$DISTRIBUTION_ID" != "YOUR_DISTRIBUTION_ID" ]; then
    aws cloudfront create-invalidation \
        --distribution-id "$DISTRIBUTION_ID" \
        --paths "/*" \
        --profile prod
    echo "CloudFront cache invalidated!"
else
    echo "WARNING: CloudFront distribution ID not set - cache not invalidated!"
    echo "Please set DISTRIBUTION_ID in this script to your CloudFront distribution ID"
fi

echo "Deployment to $BUCKET completed!"

