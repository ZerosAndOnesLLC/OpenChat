#!/bin/bash
set -e

echo "Tagging image..."
docker tag openchat-api:latest 547553741334.dkr.ecr.us-east-1.amazonaws.com/openchat-api:latest

echo "Logging in to ECR..."
aws ecr get-login-password --region us-east-1 --profile prod | docker login --username AWS --password-stdin 547553741334.dkr.ecr.us-east-1.amazonaws.com

echo "Pushing image to ECR..."
docker push 547553741334.dkr.ecr.us-east-1.amazonaws.com/openchat-api:latest

echo "Forcing ECS deployment..."
aws ecs update-service --cluster 0n1-us-east-1 --service openchat-api --force-new-deployment --profile prod --region us-east-1

echo "Deployment triggered successfully!"
