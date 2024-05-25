## RUST对接ELASTIC

### 1. 安装RUST
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. docker安装elasticsearch
```shell
docker run -d --name elasticsearch -p 9200:9200 -p 9300:9300 -e "discovery.type=single-node" elasticsearch:7.9.2
```

### 3. 添加依赖
```toml
[dependencies]
dotenv = "0.15.0"
elasticsearch = "7.9.0-alpha.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "0.2", features = ["full"] }

