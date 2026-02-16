# 创建本地数据目录
mkdir -p ~/docker-data/postgres-18

# 运行容器并挂载本地目录
docker run -d \
  --name postgres-18 \
  -e POSTGRES_USER=root \
  -e POSTGRES_PASSWORD=1234 \
  -e POSTGRES_DB=newsletter \
  -p 5432:5432 \
  -v ~/docker-data/postgres-18:/var/lib/postgresql/data \
  postgres:18



  # 创建目录
mkdir C:\docker-data\mysql -Force

# 运行容器
docker run -d `
  --name mysql-latest `
  -e MYSQL_ROOT_PASSWORD=1234 `
  -e MYSQL_DATABASE=newsletter `
  -p 3306:3306 `
  -v C:\docker-data\mysql:/var/lib/mysql `
  mysql:latest