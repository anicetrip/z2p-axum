kind: pipeline
type: docker
name: z2p-axum-ci

platform:
  os: linux
  arch: amd64

environment:
  CARGO_HOME: /opt/cargo
  RUSTUP_HOME: /opt/rustup
  CARGO_TARGET_DIR: /drone/src/target

services:
  - name: mysql
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: 1234
      MYSQL_DATABASE: newsletter
    command:
      - --default-authentication-plugin=mysql_native_password
      - --port=13306
    ports:
      - 13306

volumes:
  - name: cargo-home
    host:
      path: /var/lib/drone/cargo/home
  - name: rustup-home
    host:
      path: /var/lib/drone/cargo/rustup
  - name: cargo-target
    host:
      path: /var/lib/drone/cargo/target

steps:
  # 等待 MySQL 就绪
  - name: wait-for-mysql
    image: mysql:8.0
    commands:
      - |
        echo "Waiting for MySQL at mysql:13306 ..."
        for i in $(seq 1 120); do
          mysql -h mysql -uroot -p1234 -P 13306 -e "SELECT 1" && exit 0
          sleep 2
        done
        echo "MySQL not reachable"
        exit 1

  # 初始化 Rust
  - name: init-rustup
    image: rust:bookworm
    volumes:
      - name: cargo-home
        path: /opt/cargo
      - name: rustup-home
        path: /opt/rustup
      - name: cargo-target
        path: /drone/src/target
    commands:
      - |
        set -e
        rustup set profile minimal
        rustup toolchain install nightly --profile minimal
        rustup default nightly
        rustup show
        cargo -V
        rustc -V

 
  # 准备锁文件
  - name: prepare-cargo-lock
    image: rust:bookworm
    volumes:
      - name: cargo-home
        path: /opt/cargo
      - name: rustup-home
        path: /opt/rustup
      - name: cargo-target
        path: /drone/src/target
    commands:
      - |
        set -e
        if [ -f Cargo.lock ]; then
          cargo fetch --locked --manifest-path Cargo.toml
        else
          cargo generate-lockfile
        fi

 

  # 构建并推送 Docker 镜像到私有仓库
  - name: build-and-push
    image: gcr.io/kaniko-project/executor:debug
    environment:
      DOCKER_REGISTRY:
        from_secret: DOCKER_REGISTRY
      DOCKER_USERNAME:
        from_secret: DOCKER_USERNAME
      DOCKER_PASSWORD:
        from_secret: DOCKER_PASSWORD
      TAR_NAME: z2p-axum-${DRONE_COMMIT_SHA}.tar
    commands:
      - |
        set -e
        DEST_REPO="${DOCKER_REGISTRY}/z2p-axum"
        echo "DEST_REPO=${DEST_REPO}"

        mkdir -p /kaniko/.docker
        cat <<EOF > /kaniko/.docker/config.json
        {
          "auths": {
            "${DOCKER_REGISTRY}": {
              "username": "${DOCKER_USERNAME}",
              "password": "${DOCKER_PASSWORD}"
            }
          }
        }
        EOF

        /kaniko/executor \
          --context=dir:///drone/src \
          --dockerfile=/drone/src/Dockerfile \
          --destination=${DEST_REPO}:ci-${DRONE_COMMIT_SHA} \
          --destination=${DEST_REPO}:latest \
          --tarPath=/drone/src/${TAR_NAME}

        test -f "/drone/src/${TAR_NAME}" || { echo "image tar missing"; exit 1; }
        ls -lah "/drone/src/${TAR_NAME}"

  # 上传覆盖率报告到远程服务器
  - name: scp-coverage-to-tmp
    image: appleboy/drone-scp
    when:
      branch:
        - main
      event:
        - push
        - tag
    settings:
      host:
        from_secret: SSH_HOST
      username:
        from_secret: SSH_USER
      password:
        from_secret: SSH_PASSWORD
      port:
        from_secret: SSH_PORT
      target: /tmp
      source:
        - tarpaulin-report.html
        - lcov.info
      overwrite: true
      command_timeout: 2m

  # 上传镜像 tar 到远程服务器
  - name: scp-image-tar-to-tmp
    image: appleboy/drone-scp
    when:
      branch:
        - main
      event:
        - push
        - tag
    settings:
      host:
        from_secret: SSH_HOST
      username:
        from_secret: SSH_USER
      password:
        from_secret: SSH_PASSWORD
      port:
        from_secret: SSH_PORT
      target: /tmp
      source:
        - z2p-axum-*.tar
      overwrite: true
      command_timeout: 3m

  # 远程服务器部署
  - name: deploy
    image: appleboy/drone-ssh
    when:
      branch:
        - main
      event:
        - push
        - tag
    environment:
      PROD_DB_HOST:
        from_secret: PROD_DB_HOST
      PROD_DB_PORT:
        from_secret: PROD_DB_PORT
      PROD_DB_USER:
        from_secret: PROD_DB_USER
      PROD_DB_PASSWORD:
        from_secret: PROD_DB_PASSWORD
      PROD_DB_NAME:
        from_secret: PROD_DB_NAME
      PROD_DB_REQUIRE_SSL: "false"
    settings:
      host:
        from_secret: SSH_HOST
      username:
        from_secret: SSH_USER
      password:
        from_secret: SSH_PASSWORD
      port:
        from_secret: SSH_PORT
      envs:
        - DRONE_COMMIT_SHA
        - PROD_DB_HOST
        - PROD_DB_PORT
        - PROD_DB_USER
        - PROD_DB_PASSWORD
        - PROD_DB_NAME
        - PROD_DB_REQUIRE_SSL
      command_timeout: 12m
      script:
        - |
          set -euo pipefail

          TAR_TMP=/tmp/z2p-axum-${DRONE_COMMIT_SHA}.tar
          COV_HTML_TMP=/tmp/tarpaulin-report.html
          COV_LCOV_TMP=/tmp/lcov.info

          [ -f "$COV_HTML_TMP" ] && mv -f "$COV_HTML_TMP" ~/
          [ -f "$COV_LCOV_TMP" ] && mv -f "$COV_LCOV_TMP"  ~/
          [ -f "$TAR_TMP" ]      && mv -f "$TAR_TMP"       ~/

          if docker ps -a --format '{{.Names}}' | grep -q '^z2p-axum$'; then
            docker stop z2p-axum || true
            docker rm   z2p-axum || true
          fi

          TAR=~/z2p-axum-${DRONE_COMMIT_SHA}.tar
          docker load -i "${TAR}"

          IMAGE_ID=$(docker images --format '{{.Repository}}:{{.Tag}} {{.ID}}' \
            | awk "/z2p-axum:ci-${DRONE_COMMIT_SHA}/{print \$2}" | head -n1)
          docker tag "${IMAGE_ID}" z2p-axum:latest
          docker tag "${IMAGE_ID}" z2p-axum:${DRONE_COMMIT_SHA}

          docker run -d --name z2p-axum \
            --restart=always \
            -p 8000:8000 \
            -e APP_ENVIRONMENT=production \
            -e APP_DATABASE__HOST=${PROD_DB_HOST} \
            -e APP_DATABASE__PORT=${PROD_DB_PORT} \
            -e APP_DATABASE__USERNAME=${PROD_DB_USER} \
            -e APP_DATABASE__PASSWORD=${PROD_DB_PASSWORD} \
            -e APP_DATABASE__DATABASE_NAME=${PROD_DB_NAME} \
            -e APP_DATABASE__REQUIRE_SSL=${PROD_DB_REQUIRE_SSL} \
            z2p-axum:latest

          echo "Probing /health_check ..."
          ok=0
          for i in $(seq 1 60); do
            if curl -fsS http://127.0.0.1:8000/health_check >/dev/null; then
              echo "✅ health OK"; ok=1; break
            fi
            sleep 2
          done
          if [ "${ok}" != "1" ]; then
            echo "❌ health check failed"
            docker logs z2p-axum
            exit 1
          fi

          docker image prune -f || true
          docker container prune -f || true
          rm -f "${TAR}" || true
