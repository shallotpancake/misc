docker pull docker.all-hands.dev/all-hands-ai/runtime:0.19-nikolaik || echo "Docker command failed. Either install docker, or add yourself to the `docker` group"

docker run -d --rm --pull=always \
    -e SANDBOX_RUNTIME_CONTAINER_IMAGE=docker.all-hands.dev/all-hands-ai/runtime:0.19-nikolaik \
    -e LOG_ALL_EVENTS=true \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -v ~/.openhands-state:/.openhands-state \
    -p 3000:3000 \
    --add-host host.docker.internal:host-gateway \
    --name openhands-app \
    docker.all-hands.dev/all-hands-ai/openhands:0.19 && echo "OpenHands running at http://localhost:3000/"
