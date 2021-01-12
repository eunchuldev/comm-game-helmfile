set -e

PROFILE=$1
REGION=$2

LAST_CF_OUTPUT=$(aws cloudformation describe-stacks \
  --stack-name SaKube \
  --query "Stacks[0].Outputs" \
  --output json)

ENDPOINT_WITHOUT_SECRET=$(echo $LAST_CF_OUTPUT | jq -rc '.[] | select(.OutputKey | endswith("ArgoDbEndpoint")) | .OutputValue ')

SM_SECRET_ID=$(echo $LAST_CF_OUTPUT | jq -rc '.[] | select(.OutputKey | endswith("ArgoDbSecretManagerSecretId")) | .OutputValue ')

SECRET=$(aws secretsmanager get-secret-value --secret-id "${SM_SECRET_ID}" --query SecretString --output text)

ENDPOINT=$(python3 -c "print(''':$SECRET@'''.join(\"$ENDPOINT_WITHOUT_SECRET\".split('@')))")

echo $ENDPOINT
