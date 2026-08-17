#!/usr/bin/env bash
#
# Redeploy the app to the GCP VM.
#
#   ./deploy.sh          build images, then roll them out
#   ./deploy.sh --no-build   roll out whatever `:latest` already is
#
# Build happens on Cloud Build, not locally: the dev Macs are arm64 and the VM
# is amd64, so a locally built image would not run there.

set -euo pipefail

PROJECT=gother-price-intel
REGION=asia-southeast1
ZONE=${REGION}-a
VM=price-app
APP_DIR=/opt/price-app
COMPOSE="sudo docker compose -f docker-compose.yml -f docker-compose.prod.yml"

cd "$(dirname "$0")"

if [[ "${1:-}" != "--no-build" ]]; then
  echo "==> Building images on Cloud Build"
  gcloud builds submit --project="$PROJECT" --config=cloudbuild.yaml \
    --substitutions=SHORT_SHA="$(git rev-parse --short HEAD)" .
fi

echo "==> Syncing compose files"
gcloud compute scp docker-compose.yml docker-compose.prod.yml \
  "$VM:$APP_DIR/" --project="$PROJECT" --zone="$ZONE" --tunnel-through-iap
gcloud compute scp docker/Caddyfile \
  "$VM:$APP_DIR/docker/" --project="$PROJECT" --zone="$ZONE" --tunnel-through-iap

echo "==> Rolling out"
# `pull` before `up` so the new image is on disk first and the swap is quick.
gcloud compute ssh "$VM" --project="$PROJECT" --zone="$ZONE" --tunnel-through-iap \
  --command="cd $APP_DIR && $COMPOSE pull backend frontend && $COMPOSE up -d && sleep 15 && $COMPOSE ps"

echo "==> Verifying"
SITE="https://34-124-161-138.nip.io"
health=$(curl -s --max-time 20 "$SITE/api/health" || true)
echo "    health: $health"
# A signed-out request must still be refused — if this returns 200 the auth
# layer is not in the running image and the rollout should be rolled back.
code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 20 "$SITE/api/hotels")
echo "    /api/hotels unauthenticated: $code (expect 401)"
[[ "$code" == "401" ]] || { echo "!! auth check FAILED"; exit 1; }
echo "==> Done: $SITE"
