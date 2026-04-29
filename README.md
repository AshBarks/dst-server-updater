# DST Server Updater

Don't Starve Together 专用服务器自动更新工具。

每次运行时从 Klei 官方论坛获取正式服最新版本号，与服务器当前版本对比，若存在新版本则通过 steamcmd 自动完成更新。

## 先决条件

- 安装了 steamcmd 客户端

```bash
mkdir {你的STEAMCMD的安装目录}/steamcmd
cd {你的STEAMCMD的安装目录}/steamcmd
curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar -xvzf -
./steamcmd.sh
```

安装时如遇到依赖错误要根据错误信息安装缺失的依赖。

- 安装了 Rust 编译器

```bash
curl -fsSL https://sh.rustup.rs | sh
```

## 环境变量

| 变量名 | 必填 | 说明 |
|---|---|---|
| `DST_SERVER__ROOT` | 是 | DST 专用服务器安装目录，需存在且可读 |
| `STEAMCMD__DIR` | 是 | steamcmd 安装目录，需存在且可读 |
| `DST_UPDATER__LOG__DIR` | 是 | 日志文件输出目录，需存在且可写 |

### 示例
```bash
export DST_SERVER__ROOT=/path/to/dst_server
export STEAMCMD__DIR=/path/to/steamcmd
export DST_UPDATER__LOG__DIR=/path/to/logs
```

日志文件按天归档，命名格式为 `dst-updater-YYYY-MM-DD.log`。

## 使用方式


### 单次运行
```
cargo run
```

### 定时运行

编译项目
```
cargo build --release
```

在`target/release`目录下生成的可执行文件为`dst-server-updater`。

#### 配置定时任务

根据操作系统不同，配置不同的定时任务。以下为 Linux 示例：

```bash
crontab -e
```

在编辑器中添加以下行，每小时执行一次：
```
0 * * * * /path/to/dst-server-updater
```


## 工作流程

1. 从 Klei 论坛抓取最新 Release 版本的 build number
2. 读取 `{DST_SERVER__ROOT}/version.txt` 获取当前服务器版本
3. 比较版本号，若已是最新则退出
4. 执行 `steamcmd.sh +force_install_dir {DST_SERVER__ROOT} +login anonymous +app_update 343050 validate +quit` 完成更新
5. 检查 steamcmd 输出是否包含 `Success` 以确认更新结果
