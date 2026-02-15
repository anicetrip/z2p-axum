kind: pipeline
type: docker
name: z2p-axum-ci-check

steps:
  - name: check-env
    image: docker:24-dind
    privileged: true
    environment:
      DOCKER_REGISTRY:
        from_secret: DOCKER_REGISTRY
      DOCKER_USERNAME:
        from_secret: DOCKER_USERNAME
      DOCKER_PASSWORD:
        from_secret: DOCKER_PASSWORD
    commands:
      - |
        set -e
        echo "🔹 Checking Drone Secrets..."
        echo "DOCKER_REGISTRY: ${DOCKER_REGISTRY}"
        echo "DOCKER_USERNAME: ${DOCKER_USERNAME}"
        [ -n "${DOCKER_PASSWORD}" ] && echo "DOCKER_PASSWORD: set" || echo "DOCKER_PASSWORD: missing"

        echo "🔹 Testing Docker login..."
        echo "${DOCKER_PASSWORD}" | docker login ${DOCKER_REGISTRY} -u "${DOCKER_USERNAME}" --password-stdin

        echo "🔹 Listing repositories (requires login, won't fail if empty)..."
        docker --config /root/.docker info || echo "Docker info done"

        echo "✅ Environment and Docker connection OK"
